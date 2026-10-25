"""Repeat a School Two segment from an unchanged ordinary checkpoint.

The route retains checkpoint-gym.json and checkpoint-laboratory-approach.json
through the normal host save operation. Select the matching --resume-point;
this fixture copies those bytes into a fresh profile and uses --coop-resume.
Its result is always a segment, never a complete campaign visit.
Run in Anode; no screenshot inspection or manual save-menu input is required.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_school2 import School2Pair
from test_coop_play import ROOT, distance


class School2Resume(School2Pair):
    evidence_prefix = 'school2-resume-'
    resume_checkpoint = True

    def __init__(self, args):
        saved = args.checkpoint.read_bytes()
        payload = json.loads(saved)['payload']
        assert payload['visit']['map'] == 'skool2' and payload['visit']['epoch'] == 1
        assert payload['frame']['checkpoint_generation'] == 0
        school = payload['frame']['combat']['stage']['world']['school2']
        assert school['quest']['stage'] == 'Laboratory' and school['quest']['items']['mushroom']
        assert len(school['boojums']) >= 3 and all(b['health'] <= 0 for b in school['boojums'][:3])
        assert payload['frame']['combat']['stage']['owner'] is None
        if args.resume_point == 'laboratory-approach':
            assert all(distance(b['player']['feet'], (-64, -2060, 0)) < 110
                       for b in payload['frame']['players']), 'Not a laboratory approach checkpoint'
        super().__init__(args)
        (self.folder/'host'/'coop-checkpoint.json').write_bytes(saved)
        self.saved = payload
        self.evidence['checkpoint_sha256'] = hashlib.sha256(saved).hexdigest()
        self.evidence['checkpoint_tick'] = payload['frame']['tick']
        self.evidence['resume_point'] = args.resume_point
        source = Path(__file__).read_bytes()
        (self.folder/'resume-fixture-source.py').write_bytes(source)
        self.evidence['resume_fixture_sha256'] = hashlib.sha256(source).hexdigest()
        for role in self.commands:
            self.set(role, attack=False)

    def run(self):
        self.start_pair()
        self.wait(2)
        receipt = json.loads((self.folder/'host'/'route-resume.json').read_text())
        assert receipt == dict(build=self.build, verified=True, epoch=self.saved['visit']['epoch'],
                               tick=self.saved['frame']['tick']), receipt
        self.evidence['exact_restore'] = receipt
        for report in self.reports.values():
            assert report['visit'] == self.saved['visit']
            assert report['frame']['tick'] >= self.saved['frame']['tick']
            for current, old in zip(report['frame']['players'], self.saved['frame']['players']):
                assert distance(current['player']['feet'], old['player']['feet']) < 2
                for field in ('sanity', 'owned', 'collected', 'school_items'):
                    assert current['stats'][field] == old['stats'][field], f'Resume changed {field}'
                # The host already checked every resource exactly before the
                # first simulation tick. Live reports include ordinary passive
                # Will recovery; no attacks or collection are allowed here.
                rate, cap = {'Easy': (.3, 100), 'Normal': (.1, 10)}.get(old['stats']['difficulty'], (0, 0))
                before = old['stats']['will']
                elapsed = (report['frame']['tick'] - self.saved['frame']['tick']) / 120
                expected = min(cap, before + elapsed * rate) if before < cap else before
                assert abs(current['stats']['will'] - expected) < .002, 'Resume changed Will beyond normal recovery'
            self.verify_rejoined_world({'combat': self.saved['frame']['combat']}, report)
        self.verify_coherence()
        (self.folder/'milestone-resume.json').write_text(json.dumps(dict(
            passed=True, full_visit=False, phase='resume', build=self.build,
            first=self.args.first, scope='Exact checkpoint restore and initial client agreement only',
            internet=not self.args.local, delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms,
            evidence=self.evidence), indent=2))
        print('PASS both clients resumed unchanged checkpoint with normal passive recovery', flush=True)
        for role in self.commands:
            self.set(role, attack=True)
        if self.args.resume_point == 'laboratory-approach':
            self.laboratory_from_stairs()
        else:
            self.laboratory()
        self.greenhouse()
        self.rewards()
        assert self.arrived()
        self.verify(completed_scenes={'dice_cat', 'extendBleachers', 'Old_Gnome_Mushroom', 'OG1_Warp'})
        self.evidence['final_health'] = [b['stats']['sanity'] for b in self.bodies()]
        self.finish_pair(2)
        (self.folder/'result.json').write_text(json.dumps(dict(passed=True, full_visit=False,
            build=self.build, internet=not self.args.local, first=self.args.first,
            delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms, evidence=self.evidence,
            scope=f'Resumed School Two {self.args.resume_point}-to-exit segment only; not a complete traversal'), indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT/'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--checkpoint', type=Path, required=True)
    parser.add_argument('--resume-point', choices=('gym', 'laboratory-approach'), default='gym')
    parser.add_argument('--first', choices=('blue', 'red'), default='red')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--rejoin', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    parser.set_defaults(stage_at_altar=False, stop_after=None)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = School2Resume(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder/'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error)), indent=2))
        raise
    finally:
        (pair.folder/'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
