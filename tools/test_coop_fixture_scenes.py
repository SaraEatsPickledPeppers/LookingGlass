"""Headless regression for a scene starting between client arrival reports."""
import copy
import unittest
from types import SimpleNamespace

from test_coop_school import SchoolPair
from test_coop_school2 import verify_reunion


class SceneRace(SchoolPair):
    def __init__(self, stuck=False, pickup=True):
        self.args = SimpleNamespace(first='red')
        self.commands = {'host': None, 'guest': None}
        self.evidence = {}
        self.phase = 0
        self.started = False
        self.stuck = stuck
        self.pickup = pickup
        self.asides = []
        self.reports = {}
        for role in self.commands:
            self.reports[role] = {'combat': {'stage': {'owner': None}},
                'frame': {'tick': 0, 'players': [
                    {'warped_at': 0, 'player': {'feet': [0, 0, 0], 'grounded': True}},
                    {'warped_at': 0, 'player': {'feet': [50, 0, 0], 'grounded': True}}]}}

    def set(self, role, goal=None, *args, **kwargs):
        previous = self.commands[role]
        self.commands[role] = goal
        if role == self.first and previous is not None and goal is None and not self.started:
            self.started = True
            # Host starts the scene after the guest's earlier arrival report.
            self.reports['host']['combat']['stage']['owner'] = 'Red'

    def reached(self, role):
        return True

    def clear_departure(self, goal):
        pass

    def secure(self, **kwargs):
        pass

    def advance(self):
        self.phase += 1
        for report in self.reports.values():
            report['combat']['stage']['owner'] = 'Red' if self.stuck or self.phase == 1 else None
            report['frame']['tick'] = self.phase
            for body in report['frame']['players']:
                body['warped_at'] = 1
                body['player']['grounded'] = self.phase >= 3

    def wait(self, seconds):
        for role in self.commands:
            self.set(role)
        self.advance()

    def until(self, predicate, seconds=120):
        for _ in range(8):
            if predicate():
                return
            self.advance()
        raise AssertionError('Scene did not settle')

    def move_aside(self, role):
        assert all(r['combat']['stage']['owner'] is None for r in self.reports.values()), 'Waiting spot queried during scene'
        self.asides.append(role)

    def run_passage(self):
        self.passage([((50, 0, 0), False)], partner_done=lambda: self.pickup and self.phase >= 1)


class PassageSceneTest(unittest.TestCase):
    def test_host_scene_waits_for_both_clients_and_grounded_reunion(self):
        pair = SceneRace()
        pair.run_passage()
        self.assertGreaterEqual(pair.phase, 3)
        self.assertEqual(pair.asides, [])
        self.assertEqual(len(pair.evidence['passage_partner_completed']), 1)

    def test_scene_timeout_still_fails(self):
        with self.assertRaisesRegex(AssertionError, 'Scene did not settle'):
            SceneRace(stuck=True).run_passage()

    def test_finished_scene_does_not_replace_missing_personal_pickup(self):
        pair = SceneRace(pickup=False)
        pair.run_passage()
        self.assertEqual(pair.asides, ['guest'])
        self.assertNotIn('passage_partner_completed', pair.evidence)


class ReunionEvidenceTest(unittest.TestCase):
    def frames(self, owner='Red'):
        bodies = [dict(feet=[114, 0, 8], grounded=True, warped_at=50),
                  dict(feet=[0, 0, 0], grounded=True, warped_at=50)]
        if owner == 'Blue':
            bodies.reverse()
        return (dict(owner=owner, players=copy.deepcopy(bodies)),
                dict(owner=None, tick=50, players=copy.deepcopy(bodies)))

    def test_existing_grounded_pad_is_valid_for_either_owner(self):
        for owner in ('Blue', 'Red'):
            self.assertEqual(verify_reunion(*self.frames(owner)), 'retained')

    def test_distant_new_placement_is_rejected(self):
        old, row = self.frames()
        old['players'][0]['feet'][0] -= 10
        with self.assertRaisesRegex(AssertionError, 'displaced'):
            verify_reunion(old, row)

    def test_retained_pad_still_has_height_ground_and_distance_limits(self):
        for point, grounded in [([129, 0, 0], True), ([114, 0, 33], True), ([114, 0, 8], False)]:
            old, row = self.frames()
            for frame in (old, row):
                frame['players'][0].update(feet=point, grounded=grounded)
            with self.assertRaises(AssertionError):
                verify_reunion(old, row)

    def test_normal_placement_and_sampled_movement_remain_valid(self):
        old, row = self.frames()
        row['players'][0]['feet'] = [48, 0, 0]
        self.assertEqual(verify_reunion(old, row), 'placed')
        row['players'][0]['feet'] = [114, 0, 8]
        row['tick'] += 1
        self.assertEqual(verify_reunion(old, row), 'placed')

    def test_mismatched_relocation_ticks_are_rejected(self):
        old, row = self.frames()
        row['players'][0]['warped_at'] -= 1
        with self.assertRaisesRegex(AssertionError, 'both presentation tracks'):
            verify_reunion(old, row)


if __name__ == '__main__':
    unittest.main()
