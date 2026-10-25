"""Exercise School Two's upper-gym jump with two independent clients.

Only the initial spawn is staged. Players use the real lever and bleachers,
jump through ordinary inputs, and settle on the next platform. This focused
component does not certify the gym quest or a full campaign visit. Run in Anode.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_school2 import School2Pair
from test_coop_play import ROOT, distance


class GymJumpPair(School2Pair):
    evidence_prefix = 'gym-jump-'
    start_at = (2150, -2080, 0)

    def __init__(self, args):
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder / 'gym-jump-fixture-source.py').write_bytes(source)
        self.evidence['gym_jump_fixture_sha256'] = hashlib.sha256(source).hexdigest()
        self.evidence['staged_initial_spawn'] = self.start_at
        for role in self.commands:
            self.set(role, attack=False)

    def run(self):
        self.start_pair()
        self.wait(1)
        self.go((2150, -2000, 0))
        self.gym_ascent()
        # A waiting point must stay clear for a whole five-second swing, not
        # merely happen to be free when the first player arrives. The mover
        # safely stops against a body, so health alone cannot detect this stall.
        def pendulum_time():
            return next(o['time'] for o in self.school()['objects'] if o['name'] == 'gym_pend1')
        before = pendulum_time()
        tick = self.reports['host']['frame']['tick']
        self.until(lambda: self.reports['host']['frame']['tick'] >= tick + 600, 15)
        advanced = pendulum_time() - before
        assert advanced >= 4.9, f'Waiting players blocked the gym lamp: only {advanced:.3f}s of motion'
        self.evidence['gym_departure'] = dict(goal=self.gym_exit, lamp_advance_seconds=advanced)
        for report in self.reports.values():
            assert report['visit']['map'] == 'skool2' and report['visit']['epoch'] == 1
            for index,body in enumerate(report['frame']['players']):
                assert body['player']['grounded']
                assert abs(body['player']['feet'][2] - 440) < 2
                first_index = 0 if self.first == 'host' else 1
                goal = self.gym_exit[:2] if index == first_index else (1600, -2275)
                assert distance(body['player']['feet'][:2], goal) < 100
        for rows in self.history.values():
            assert all(row['checkpoint_generation'] == 0 for row in rows)
            assert all(body['health'] == 100 and body['spawned_at'] == 0
                       for row in rows for body in row['players']), 'Jump caused damage or respawn'
            # A walk/fall is not a passing substitute for the planned jump.
            for index in range(2):
                assert any(row['players'][index]['jumps'] > rows[0]['players'][index]['jumps']
                           for row in rows), 'Player did not jump'
        assert self.captures == {'extendBleachers'}
        for rows in self.history.values():
            assert {row['owner'] for row in rows if row['scene'] == 'extendBleachers'} == {self.args.first.title()}
        assert all(r['combat']['stage']['world']['gym']['used'] for r in self.reports.values())
        self.verify_coherence()
        self.evidence['final_health'] = [b['stats']['sanity'] for b in self.bodies()]
        self.finish_pair(1)
        (self.folder / 'result.json').write_text(json.dumps(dict(
            passed=True, build=self.build, internet=not self.args.local, first=self.args.first,
            delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms, evidence=self.evidence,
            scope='Staged gym lever, bleacher ascent, jump and supported settling for both players; not gym quest or full traversal'), indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    parser.set_defaults(stage_at_altar=False, stop_after=None, rejoin=False)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = GymJumpPair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error)), indent=2))
        raise
    finally:
        (pair.folder / 'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
