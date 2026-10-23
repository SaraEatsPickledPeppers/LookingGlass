"""Two real clients walk into the authored school theatre and reunite afterward.

This starts near that trigger; it is not a school traversal/completion test.
Run in Anode. Internet transport is the default; --local is for faster diagnosis.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time
import uuid

from test_coop_play import ROOT, read, distance


def verify(history, first):
    assert history, 'No playable state observed'
    actor = 0 if first == 'blue' else 1
    partner = 1 - actor
    active = [r for r in history if r['owner'] is not None]
    assert len(active) > 20, 'Original scene was not observed over time'
    assert all(r['owner'] == first.title() for r in active), 'Wrong scene owner'
    waiting = active[0]['players'][partner]['feet']
    assert all(distance(r['players'][partner]['feet'], waiting) < .01 for r in active), 'Waiting partner moved during scene'
    assert max(distance(r['camera'], active[0]['camera']) for r in active) > 100, 'Original scene camera did not move'
    reunited = [r for r in history if r['owner'] is None and all(b['warped_at'] > 0 for b in r['players'])]
    assert reunited, 'Both players did not reunite after the scene'
    first_reunion = reunited[0]
    players = first_reunion['players']
    assert players[0]['warped_at'] == players[1]['warped_at']
    assert all(b['spawned_at'] == 0 and b['health'] > 0 for b in players), 'Reunion was misreported as death/respawn'
    separation = distance(players[0]['feet'], players[1]['feet'])
    assert 40 <= separation <= 110, f'Reunion separation {separation}'
    assert all(r['checkpoint_generation'] == 0 for r in history), 'Scene unexpectedly wiped'
    return {'reunion_tick': players[0]['warped_at'], 'separation': separation,
            'scene_first_tick': active[0]['tick'], 'scene_last_tick': active[-1]['tick']}


def run(args):
    folder = ROOT / 'private' / 'coop' / ('scene-test-' + uuid.uuid4().hex)
    folder.mkdir(parents=True)
    invite = folder / 'invite.txt'
    history = {role: [] for role in ('host', 'guest')}
    processes, logs = [], []
    flags = subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0
    try:
        def start(role):
            profile = folder / role
            profile.mkdir()
            log = (folder / (role + '.log')).open('w')
            logs.append(log)
            command = [str(args.exe.resolve()), '--data', str(args.data.resolve()), '--map', 'skool1', '--no-audio',
                       '--profile', str(profile), '--coop-host' if role == 'host' else '--coop-join', str(invite),
                       '--coop-scene-check', args.first, '--coop-delay-ms', str(args.delay_ms), '--coop-jitter-ms', str(args.jitter_ms)]
            if role == 'host' and args.local:
                command.append('--coop-local')
            p = subprocess.Popen(command, cwd=ROOT, stdout=log, stderr=subprocess.STDOUT, creationflags=flags)
            processes.append(p)
            return p
        host = start('host')
        deadline = time.monotonic() + 150
        while not invite.exists():
            assert host.poll() is None, (folder / 'host.log').read_text()
            assert time.monotonic() < deadline, 'Invite timeout'
            time.sleep(.1)
        start('guest')
        deadline = time.monotonic() + 240
        while any(p.poll() is None for p in processes):
            assert time.monotonic() < deadline, 'Scene timeout'
            for role, process in zip(history, processes):
                assert process.poll() in (None, 0), (folder / (role + '.log')).read_text()
            for role in history:
                report = read(folder / role / 'coop-live.json')
                if not report or not report['playing']:
                    continue
                frame, stage = report['frame'], report['combat']['stage']
                if history[role] and history[role][-1]['tick'] == frame['tick']:
                    continue
                history[role].append({'tick': frame['tick'], 'checkpoint_generation': frame['checkpoint_generation'],
                    'owner': stage['owner'], 'camera': report['camera'], 'target': report['camera_target'],
                    'players': [{'feet': p['player']['feet'], 'health': p['stats']['sanity'],
                        'spawned_at': p['spawned_at'], 'warped_at': p['warped_at']} for p in frame['players']]})
            time.sleep(.05)
        for role, process in zip(history, processes):
            assert process.returncode == 0, (folder / (role + '.log')).read_text()
            for capture in ('coop-scene.png', 'coop-screenshot.png'):
                assert (folder / role / capture).exists(), f'Missing {role} {capture}'
        results = {role: verify(reports, args.first) for role, reports in history.items()}
        assert results['host']['reunion_tick'] == results['guest']['reunion_tick'], 'Guest did not see the authoritative reunion'
        assert abs(results['host']['separation'] - results['guest']['separation']) < .1
        (folder / 'result.json').write_text(json.dumps({'passed': True, 'first': args.first,
            'internet': not args.local, 'delay_ms': args.delay_ms, 'jitter_ms': args.jitter_ms,
            'results': results, 'scope': 'school theatre fixture only; campaign not validated'}, indent=2))
        print('PASS two rendered clients: first-arrival theatre ownership, single scene camera, frozen partner, safe reunion.')
        print('Evidence:', folder)
    finally:
        (folder / 'history.json').write_text(json.dumps(history, indent=2))
        for p in processes:
            if p.poll() is None:
                subprocess.run(['taskkill', '/PID', str(p.pid), '/T', '/F'], capture_output=True)
                p.wait(timeout=10)
        for log in logs:
            log.close()
        invite.unlink(missing_ok=True)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/debug/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=0)
    parser.add_argument('--jitter-ms', type=int, default=0)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Network test delays must be between 0 and 500 milliseconds')
    run(args)
