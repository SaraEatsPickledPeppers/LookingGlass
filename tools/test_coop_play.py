"""Run two rendered co-op movement clients in a desktop session.

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


def run(exe, data, local):
    folder = ROOT / 'private' / 'coop' / ('play-test-' + uuid.uuid4().hex)
    folder.mkdir(parents=True)
    invite = folder / 'invite.txt'
    flags = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0
    processes = []
    logs = []
    initial = {}
    latest = {}
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
        assert all(d > 5 for d in moved), f'Both independently driven players must move: {moved}'
        assert all(distance(a, b) < 1 for a, b in zip(after, replicated)), 'Guest diverged from the host'
        assert latest['guest']['frame']['acknowledged_input'] > 0, 'No guest controls acknowledged'
        assert distance(latest['host']['camera'], latest['guest']['camera']) > 10, 'Cameras are not independent'
        for role in ('host', 'guest'):
            assert (folder / role / 'coop-screenshot.png').exists(), 'Missing rendered capture'
        result = {'passed': True, 'internet': not local, 'moved': moved,
                  'initial': initial, 'final': latest, 'scope': 'two-player movement and replication; campaign not validated'}
        (folder / 'result.json').write_text(json.dumps(result, indent=2))
        print('PASS two rendered processes: independent movement/cameras and matching host/guest state.')
        print('Evidence:', folder)
    finally:
        (folder / 'diagnostic.json').write_text(json.dumps({'initial': initial, 'latest': latest,
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
    args = parser.parse_args()
    run(args.exe.resolve(), args.data.resolve(), args.local)
