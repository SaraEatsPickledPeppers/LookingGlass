"""Run two rendered co-op movement/combat clients in a desktop session.

This is a development gameplay slice, not a campaign completion check. Run in
Anode to leave the user's desktop free. Public internet hosting is the default.
"""
import argparse
import json
import math
import os
from pathlib import Path
import subprocess
import time
import uuid

ROOT = Path(__file__).resolve().parents[1]


def read(path):
    try:
        return json.loads(path.read_text())
    except (OSError, ValueError):
        return None


def distance(a, b):
    return math.sqrt(sum((x - y) ** 2 for x, y in zip(a, b)))


def run(exe, data, local, combat=False):
    folder = ROOT / 'private' / 'coop' / ('play-test-' + uuid.uuid4().hex)
    folder.mkdir(parents=True)
    invite = folder / 'invite.txt'
    flags = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0
    processes = []
    logs = []
    initial = {}
    latest = {}
    minimum_health = {role: [100., 100.] for role in ('host', 'guest')}
    try:
        def start(role):
            profile = folder / role
            profile.mkdir()
            log = (folder / (role + '.log')).open('w')
            logs.append(log)
            args = [str(exe), '--data', str(data), '--map', 'skool1', '--no-audio',
                    '--profile', str(profile), '--coop-host' if role == 'host' else '--coop-join',
                    str(invite), '--coop-drive']
            if local and role == 'host':
                args.append('--coop-local')
            if combat and role == 'host':
                args.append('--combat-preview')
            process = subprocess.Popen(args, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, creationflags=flags)
            processes.append(process)
            return process
        host = start('host')
        deadline = time.monotonic() + 150
        while not invite.exists():
            assert host.poll() is None, (folder / 'host.log').read_text()
            assert time.monotonic() < deadline, 'Host invite timeout'
            time.sleep(.05)
        guest = start('guest')
        deadline = time.monotonic() + 120
        while any(p.poll() is None for p in processes):
            assert time.monotonic() < deadline, 'Gameplay test timeout'
            for role in ('host', 'guest'):
                report = read(folder / role / 'coop-live.json')
                if report and report['playing']:
                    initial.setdefault(role, report)
                    latest[role] = report
                    minimum_health[role] = [min(old, body['stats']['sanity']) for old, body in
                        zip(minimum_health[role], report['frame']['players'])]
            time.sleep(.05)
        assert host.returncode == 0, (folder / 'host.log').read_text()
        assert guest.returncode == 0, (folder / 'guest.log').read_text()
        assert set(latest) == {'host', 'guest'}, 'Both clients must receive a playing state'
        assert latest['host']['pid'] != latest['guest']['pid']
        assert latest['host']['camera_player'] == 'Blue'
        assert latest['guest']['camera_player'] == 'Red'
        positions = lambda r: [p['player']['feet'] for p in r['frame']['players']]
        before = positions(initial['host'])
        after = positions(latest['host'])
        replicated = positions(latest['guest'])
        moved = [distance(a, b) for a, b in zip(before, after)]
        if combat:
            guard = lambda r: r['combat']['npcs']['actors'][0]['guard']['health']
            assert guard(initial['host']) > 0, 'Encounter did not begin with a living guard'
            assert guard(latest['host']) == guard(latest['guest']) == 0, 'Both clients must see the defeated guard'
            # Defeating the guard creates an essence pickup which can heal the
            # victim before the final frame. Verify the actual fight history.
            health = minimum_health['host']
            assert 0 < min(health) < 100 and max(health) == 100, f'Enemy damage was not personal: {health}'
            assert minimum_health['guest'] == health, 'Guest did not see the same personal damage'
            assert all(c['projectiles']['shots'] > 0 for c in latest['host']['combat']['characters']), 'Both players must attack through their real weapon path'
        else:
            assert all(d > 5 for d in moved), f'Both independently driven players must move: {moved}'
        assert all(distance(a, b) < 1 for a, b in zip(after, replicated)), 'Guest diverged from the host'
        assert latest['guest']['frame']['acknowledged_input'] > 0, 'No guest controls acknowledged'
        cameras = initial if combat else latest
        # The school back wall retracts both camera arms to the same Y plane.
        # Their origins remain distinct, but can be only a few units apart even
        # when they follow players standing 48 units apart. Role checks above
        # also verify that each camera watches its own body.
        assert distance(cameras['host']['camera'], cameras['guest']['camera']) > .1, 'Both clients used the same camera origin'
        for role in ('host', 'guest'):
            assert (folder / role / 'coop-screenshot.png').exists(), 'Missing rendered capture'
        result = {'passed': True, 'internet': not local, 'combat': combat, 'moved': moved,
                  'initial': initial, 'final': latest, 'minimum_health': minimum_health,
                  'scope': 'native enemy, weapon and personal damage; campaign not validated' if combat else 'two-player movement and replication; campaign not validated'}
        (folder / 'result.json').write_text(json.dumps(result, indent=2))
        print('PASS two rendered processes:', 'shared combat and personal damage.' if combat else 'independent movement/cameras and matching host/guest state.')
        print('Evidence:', folder)
    finally:
        (folder / 'diagnostic.json').write_text(json.dumps({'initial': initial, 'latest': latest, 'minimum_health': minimum_health,
            'exit_codes': [p.poll() for p in processes]}, indent=2))
        for process in processes:
            if process.poll() is None:
                if os.name == 'nt':
                    subprocess.run(['taskkill', '/PID', str(process.pid), '/T', '/F'], capture_output=True)
                else:
                    process.terminate()
                process.wait(timeout=10)
        for log in logs:
            log.close()
        invite.unlink(missing_ok=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/debug/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--combat', action='store_true', help='Staged school guard, using real Blade attacks')
    args = parser.parse_args()
    run(args.exe.resolve(), args.data.resolve(), args.local, args.combat)
