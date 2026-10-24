"""Exercise spiral-lift boarding from a staged lower-library start.

Two independent clients use the real internet connection and ordinary movement.
Only the initial spawn is staged, through --start-at; no live positions, health,
quest state or platform state are written. This is a component test, not school
traversal evidence. Run inside Anode or another dedicated desktop session.
"""
import argparse
import hashlib
import json
from pathlib import Path

from test_coop_school import SchoolPair
from test_coop_play import ROOT


class SchoolLiftPair(SchoolPair):
    evidence_prefix = 'school-lift-'
    start_at = (144, 4230, -288)

    def __init__(self, args):
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder / 'lift-fixture-source.py').write_bytes(source)
        self.evidence['lift_fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def run(self):
        self.start_pair()
        self.wait(1)
        self.go(self.start_at)
        self.lift('library_spiral_lift', (144, 4360, -264), (192, 4360, -264), 64)
        self.verify_coherence()
        self.evidence['final_health'] = [b['stats']['sanity'] for b in self.bodies()]
        assert all(r['visit']['map'] == 'skool1' and r['visit']['epoch'] == 1
                   for r in self.reports.values()), 'Lift test unexpectedly changed level'
        assert all(b['spawned_at'] == 0 for r in self.history.values() for row in r for b in row['players'])
        self.finish_pair(1)
        (self.folder / 'result.json').write_text(json.dumps(dict(
            passed=True, build=self.build, internet=not self.args.local,
            first=self.args.first, delay_ms=self.args.delay_ms, jitter_ms=self.args.jitter_ms,
            evidence=self.evidence,
            scope='Staged spiral-lift approach, paired boarding and ride only; not campaign traversal'), indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--first', choices=('blue', 'red'), default='blue')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = SchoolLiftPair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder / 'failed-result.json').write_text(json.dumps(dict(passed=False, error=str(error)), indent=2))
        raise
    finally:
        (pair.folder / 'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
