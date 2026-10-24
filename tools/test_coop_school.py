"""Traverse the first school visit through two independent internet clients.

The authored theatre, library, flying books, lifts and recipe exit are reached
with ordinary movement/attack inputs. No positions, resources or quest state
are written. Death, focused combat and optional secret checks remain separate.
"""
import argparse
import copy
import hashlib
import json
import time
from pathlib import Path

from test_coop_pandemonium import PandemoniumPair
from test_coop_play import ROOT, distance


class SchoolPair(PandemoniumPair):
    map_name = 'skool1'
    next_map = 'skool2'
    evidence_prefix = 'school-route-'

    def __init__(self,args):
        self.middle_commands = {}
        self.middle_done = set()
        super().__init__(args)
        source = Path(__file__).read_bytes()
        (self.folder/'fixture-source.py').write_bytes(source)
        self.evidence['fixture_sha256'] = hashlib.sha256(source).hexdigest()

    def set(self,role,*args,runup=None,speed_limit=None,**kwargs):
        super().set(role,*args,**kwargs)
        self.commands[role]['input_lead'] = min(1.5,.05+2*(self.args.delay_ms+self.args.jitter_ms)/1000) if role == 'guest' else 0
        self.commands[role]['runup'] = runup
        self.commands[role]['speed_limit'] = speed_limit

    def school(self,role='host'):
        return self.reports[role]['combat']['stage']['world']['school']

    def pump(self):
        if (self.folder/'stop-test').exists():
            raise RuntimeError('School test stopped; partial evidence retained')
        super().pump()
        for role,report in self.reports.items():
            if self.history[role]:
                self.history[role][-1]['school'] = report['combat']['stage']['world']['school']
        if len(self.reports) == 2 and not self.arrived():
            scene = self.reports['host']['route_driver']['scene']
            if scene and scene == self.reports['guest']['route_driver']['scene']:
                threshold = 20 if scene == 'Theatre_Cinematic' else 2.5
                if scene not in self.middle_commands and all(
                        self.school(r)['first_cinema']['time'] >= threshold for r in self.reports):
                    pending = {}
                    for role,old in list(self.commands.items()):
                        self.set(role,old['goal'],old['navigate'],capture=True,
                                 horizontal=old['horizontal'],run=old['run'],jump=old['jump'])
                        pending[role] = self.commands[role]['id']
                    self.middle_commands[scene] = pending
            for scene,pending in self.middle_commands.items():
                if scene not in self.middle_done and all((self.folder/r/f'route-capture-{i}.png').exists()
                                                         for r,i in pending.items()):
                    self.middle_done.add(scene)
                    self.evidence.setdefault('middle_captures',{})[scene] = pending

    def reached(self,role):
        return super().reached(role) and self.bodies(role)[0 if role == 'host' else 1]['player']['grounded']

    def until(self,predicate,seconds=120):
        end = time.monotonic()+seconds
        while not predicate():
            assert time.monotonic() < end, 'School timeout: '+str({role:dict(
                tick=r['frame']['tick'],phase=r['route_driver']['phase'],command=self.commands[role],
                feet=[b['player']['feet'] for b in r['frame']['players']]) for role,r in self.reports.items()})
            self.pump()

    def secure(self):
        if self.arrived():
            return
        self.wait(.5)
        def safe():
            for r in self.reports.values():
                if any(e['active'] and e['health'] > 0 and any(v and d < 700
                       for v,d in zip(e['line_of_sight'],e['distance']))
                       for e in r['route_driver']['guards']+r['route_driver']['boojums']):
                    return False
                cast = r['combat']['stage']['world']['encounters']
                if cast and any(a['enemy'].get('Boojum',{}).get('waves') for a in cast['actors']):
                    return False
            return True
        self.until(safe,60)

    def legs(self,goals):
        for goal,navigate in goals:
            self.go(goal,navigate)
            self.secure()

    def lift(self,name,first_goal,second_goal,top):
        def obj(role='host'):
            return next(o for o in self.school(role)['objects'] if o['name'] == name)
        self.set(self.partner)
        self.set(self.first,first_goal,navigate=True)
        self.until(lambda: self.reached(self.first),40)
        self.set(self.first)
        initial = {r:copy.deepcopy(obj(r)) for r in self.reports}
        clocks = {r:self.school(r)['clock'] for r in self.reports}
        self.until(lambda: all(self.school(r)['clock'] >= t+2 for r,t in clocks.items()),10)
        for role,before in initial.items():
            assert obj(role)['pose'] == before['pose'], 'Lift left before both players boarded'
            assert obj(role)['motion'] == before['motion'], 'Lift timer advanced while partner absent'
        self.capture_pair(name+'-waiting')
        self.set(self.partner,second_goal,navigate=True)
        index = 0 if self.partner == 'host' else 1
        self.until(lambda: all(distance(self.bodies(r)[index]['player']['feet'][:2],second_goal[:2]) < 24
                               and self.bodies(r)[index]['player']['grounded'] for r in self.reports),40)
        self.set(self.partner)
        self.until(lambda: all(b['player']['feet'][2] >= top-1 and b['player']['grounded']
                               for r in self.reports for b in self.bodies(r)),40)
        self.capture_pair(name+'-top')
        self.evidence.setdefault('lifts',[]).append(dict(name=name,waiting_seconds=2,
            both_reached_height=top,first=self.args.first))
        print('PASS school paired boarding and ride:',name,flush=True)

    def lower_library_landing(self):
        # A full-speed, timed plan can overshoot this narrow shelf under jitter.
        # Walk slowly off its edge, then steer back toward the lower shelf from
        # observed motion. These are ordinary inputs, including real fall damage.
        for role in (self.first,self.partner):
            other = 'guest' if role == 'host' else 'host'
            index = 0 if role == 'host' else 1
            self.set(other)
            self.set(role,(56,2780,384))
            self.until(lambda: self.reached(role),25)
            self.set(role,(96,2780,384))
            self.until(lambda: self.bodies(role)[index]['player']['feet'][2] < 382,15)
            self.set(role,(-32,2780,64))
            self.until(lambda: self.reached(role),20)
            assert abs(self.bodies(role)[index]['player']['feet'][2]-64) < 2, 'Missed the lower library shelf'
            self.set(role)
        self.capture_pair('lower-library-landing')
        self.secure()

    def mallet_landing(self):
        # The altar is across the stairwell, not along a continuous balcony.
        # Stop on the lower stair before the short jump to the altar platform.
        self.legs([((-2304,2624,-342),True),((-2304,2544,-352),False),
                   ((-2272,2464,-256),True)])

    def checkpoint_stop(self,label):
        if label not in getattr(self.args,'checkpoint_stops',[]):
            return
        for role in self.commands:
            self.set(role)
        self.wait(.5)
        (self.folder/f'ready-for-{label}.json').write_text(json.dumps(dict(
            tick=self.reports['host']['frame']['tick'],build=self.build)))
        print(f'Ready for ordinary menu checkpoint at {label}; create continue-{label} to proceed.',flush=True)
        self.until(lambda: (self.folder/f'continue-{label}').exists(),180)

    def balcony_gap(self):
        # A short running approach followed by feedback steering is less brittle
        # than replaying a timed sequence of sideways inputs across this gap.
        for role in (self.first,self.partner):
            # The delayed guest aims farther back while braking; its pending
            # southward inputs still carry it toward the centre of the shelf.
            goal = (-1664,3040 if role == 'host' else 3072,-256)
            index = 0 if role == 'host' else 1
            jumps = self.bodies()[index]['player']['jumps']
            self.set(role,goal,run=True,jump=True,runup=dict(millis=200,wish=[0,-1]))
            # Air steering takes time to shed running speed. Include braking
            # time as well as network delivery when aiming at this short ledge.
            self.commands[role]['input_lead'] += .30
            self.until(lambda: self.bodies()[index]['player']['jumps'] > jumps
                       and not self.bodies()[index]['player']['grounded']
                       and self.bodies()[index]['player']['velocity'][2] < 0,8)
            # Release run before touchdown: grounded acceleration is much
            # stronger and a delayed running correction can cross the ledge.
            self.set(role,goal)
            def landed():
                body = self.bodies()[index]['player']
                return (body['jumps'] > jumps and body['grounded'] and
                        abs(body['feet'][2]-goal[2]) < 20 and distance(body['feet'],goal) < 45)
            self.until(landed,20)
            self.set(role)
            self.wait(.5)
            assert landed(), f'{role}: did not remain on the balcony landing'
        self.capture_pair('balcony-gap')
        self.secure()

    def theatre_healing(self):
        self.until(lambda: all(next(g for g in r['route_driver']['guards']
                                   if g['name'] == 'play_guard1')['health'] == 0
                               for r in self.reports.values()),20)
        drops = self.bodies()[0]['stats']['loot']['skool1$first']['drops']
        reward = next(d for d in drops if d['id'] == 3000019)
        self.go(reward['origin'])
        self.capture_pair('theatre-earned-healing')

    def theatre(self,upstairs=True):
        self.legs([((-1888,2290,-512),False),((-2340,2368,-480),True)])
        self.set(self.partner)
        self.set(self.first,(-2560,2368,-480))
        self.until(lambda: all(r['combat']['stage']['owner'] is not None for r in self.reports.values()),30)
        self.until(lambda: all(r['combat']['stage']['owner'] is None for r in self.reports.values()),120)
        self.capture_pair('theatre-reunion')
        # The original scene lands both players on the lower theatre floor.
        # Align with the stair opening before walking east, instead of spending
        # a minute sliding diagonally along its vertical side wall.
        # Complete the stairs before waiting for the upper guard: their side
        # blocks Blade throws from this low alignment point.
        self.go((-3336,2368,-672))
        self.theatre_healing()
        if not upstairs:
            return
        self.go((-3336,2368,-672))
        self.legs([((-2560,2368,-480),False),((-2340,2368,-480),True),
                   ((-2256,2544,-480),False),((-2304,2624,-342),True),
                   ((-2304,2800,-256),True)])
        self.capture_pair('theatre-and-shelf-reunion')

    def route(self):
        self.wait(1)
        self.capture_pair('school-entrance')
        self.theatre()
        self.mallet_landing()
        # The essence beside the Mallet belongs to the return visit; the first
        # visit uses the fresh theatre reward above for ordinary healing.
        self.legs([((-2274,2334,-256),False),((-2272,2440,-256),False),
                   ((-2304,2544,-352),True),((-2304,2624,-342),True),((-2304,2800,-256),True),
                   ((-2304,3248,-256),False),((-1664,3248,-256),False)])
        self.checkpoint_stop('balcony')
        self.balcony_gap()
        self.library_route()

    def library_route(self):
        self.legs([((-1480,3008,-256),True),
                   ((-208,3008,-256),False),((-208,3456,-256),False),
                   ((-208,3968,-256),False),((208,3968,-288),False),
                   ((208,3520,-288),False),((700,3520,-264),True),
                   ((1000,3360,-256),False),((944,3440,-256),False),
                   ((700,3440,-256),False),((208,3440,-288),False),
                   ((208,4160,-264),False),((144,4230,-288),False)])
        self.lift('library_spiral_lift',(144,4360,-264),(192,4360,-264),64)
        self.legs([((272,4160,64),False),((700,4096,64),False),((884,4056,64),False),
                   ((0,4096,64),False),((0,3500,64),False),((-140,3300,64),False),
                   ((208,3300,400),True),((-32,3496,384),True),((-464,3520,384),False),
                   ((-464,3800,384),False),((-464,4096,384),False),((-276,4160,384),False)])
        self.lift('elevator2',(-276,4360,400),(-228,4360,400),696)
        self.legs([((-276,4160,704),False),((-212,3860,704),False),((-64,3968,704),False),
                   ((-64,3520,704),False),((-160,3520,704),False)])
        self.wait(8.5)
        self.capture_pair('flying-books-bridge')
        self.checkpoint_stop('bridge')
        self.book_bridge()

    def book_jump(self,approach,goal):
        self.go(approach)
        for role in (self.first,self.partner):
            index = 0 if role == 'host' else 1
            jumps = self.bodies()[index]['player']['jumps']
            self.set(role,goal,run=True,jump=True,speed_limit=220)
            def landed():
                p = self.bodies()[index]['player']
                return (p['jumps'] > jumps and p['grounded'] and
                        abs(p['feet'][2]-goal[2]) < 5 and distance(p['feet'],goal) < 60)
            self.until(landed,15)
            self.set(role,goal)
            self.until(lambda: self.reached(role),10)
            self.set(role)
            self.wait(.5)
            assert landed(), f'{role}: left the book after landing'
        self.capture_pair('book-landing-'+str(goal[1]))

    def book_bridge(self):
        # The first book joins this landing with a four-unit step. Walk onto it
        # directly; a jump search from this narrow edge can spend its entire
        # search budget exploring the shelves below the bridge.
        self.go((-150,3396,708))
        self.book_jump((-130,3334,708),(-40,3186,724))
        self.book_jump((20,3158,724),(136,3014,716))
        self.book_jump((160,2990,716),(224,2796,716))
        self.legs([((204,2450,704),True)])
        self.wait(4)
        self.capture_pair('recipe-reunion')
        if self.args.rejoin:
            self.rejoin_guest()
        self.checkpoint_stop('descent')
        self.descent()

    def descent(self):
        self.legs([((104,2736,624),True),((0,2824,564),True),((-32,2780,384),True)])
        self.lower_library_landing()
        self.legs([((200,2776,-288),True)])
        self.until(self.arrived,90)

    def verify_rejoined_world(self,before,report):
        old = before['combat']['stage']['world']['school']
        new = report['combat']['stage']['world']['school']
        for field in ('books','returning','theatre','library','shelves','recipe','secret_open','secret_claimed'):
            assert new[field] == old[field], f'Rejoin changed school {field}'
        assert new['first_cinema']['beat'] == old['first_cinema']['beat']

    def verify_coherence(self):
        by_tick = {(r['epoch'],r['tick']):r for r in self.history['host']}
        compared = 0
        for row in self.history['guest']:
            other = by_tick.get((row['epoch'],row['tick']))
            if other:
                for field in ('players','owner','school','encounters'):
                    assert row[field] == other[field], f'Authoritative school {field} differs'
                compared += 1
        assert compared > 0, 'No coincident full school reports were observed'
        self.evidence['matching_full_school_frames'] = compared
        self.verify_network_states()

    def verify(self):
        expected = {'Theatre_Cinematic','Skool1_OG_MoveShelf','shelf_cinematic',
                    'Skool1_Book_Win','book_cinematic','Book_Ingredients_Exit'}
        assert self.captures == expected, self.captures
        for role,rows in self.history.items():
            visit = [r for r in rows if r['epoch'] == 1]
            owners = {scene:{r['owner'] for r in visit if r['scene'] == scene} for scene in expected}
            assert all(ids == {self.args.first.title()} for ids in owners.values()), owners
            reunions = set()
            for old,row in zip(visit,visit[1:]):
                if old['owner'] is not None and row['owner'] == old['owner']:
                    other = 1 if row['owner'] == 'Blue' else 0
                    assert row['players'][other]['feet'] == old['players'][other]['feet'], 'Partner moved during school scene'
                if old['owner'] is not None and row['owner'] is None:
                    a,b = row['players']
                    assert a['warped_at'] == b['warped_at'] > 0
                    assert distance(a['feet'],b['feet']) <= 110+max(0,row['tick']-a['warped_at'])*640/120
                    reunions.add(a['warped_at'])
            assert all(b['spawned_at'] == 0 for r in rows for b in r['players'])
            assert any(r['school']['books'] == ['Bridge']*4 for r in visit)
            assert any(r['school']['recipe'] for r in visit)
            assert all('skool1:78' in b['stats']['collected'] for b in self.bodies(role)), 'Personal Mallet pickup missing'
            assert self.reports[role]['visit']['entry'] == 'skool2_start1'
            self.evidence[role] = dict(owners={s:sorted(ids) for s,ids in owners.items()},reunions=sorted(reunions))
        self.verify_coherence()

    def run(self):
        self.start_pair()
        self.route()
        self.verify()
        self.finish_pair(2)
        (self.folder/'result.json').write_text(json.dumps(dict(passed=True,build=self.build,
            internet=not self.args.local,first=self.args.first,delay_ms=self.args.delay_ms,
            jitter_ms=self.args.jitter_ms,evidence=self.evidence,
            scope='First school traversal, scenes, paired lifts and recipe exit; focused combat and recovery remain separate'),indent=2))


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe',type=Path,default=ROOT/'target/debug/looking-glass.exe')
    parser.add_argument('--data',type=Path,required=True)
    parser.add_argument('--first',choices=('blue','red'),default='red')
    parser.add_argument('--local',action='store_true')
    parser.add_argument('--rejoin',action='store_true')
    parser.add_argument('--checkpoint-stops',nargs='*',choices=('balcony','bridge','descent'),default=[],
                        help='Hold neutral inputs for up to 180 seconds for a normal menu checkpoint; create continue-<stop> in the evidence folder to resume')
    parser.add_argument('--delay-ms',type=int,default=40)
    parser.add_argument('--jitter-ms',type=int,default=20)
    args = parser.parse_args()
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    pair = SchoolPair(args)
    try:
        pair.run()
    except Exception as error:
        (pair.folder/'failed-result.json').write_text(json.dumps(dict(passed=False,error=str(error)),indent=2))
        raise
    finally:
        (pair.folder/'last-reports.json').write_text(json.dumps(pair.reports))
        pair.close()
