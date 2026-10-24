"""Run focused co-op regressions and write a compact machine-readable report.

Run in Anode when selecting rendered tests (presence or any school route). Native tests
can run without a desktop. Screenshots remain in private evidence folders and
are never opened by this runner. A pass is not full campaign certification.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import signal
import subprocess
import sys
import time
import uuid

ROOT = Path(__file__).resolve().parents[1]
EVIDENCE = ROOT / 'private' / 'coop'


def digest(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


def stop_owned_process(process):
    if process.poll() is not None:
        return
    if os.name == 'nt':
        subprocess.run(['taskkill', '/PID', str(process.pid), '/T', '/F'],
                       stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, check=True)
    else:
        os.killpg(process.pid, signal.SIGTERM)
    try:
        process.wait(timeout=10)
    except subprocess.TimeoutExpired:
        if os.name != 'nt':
            os.killpg(process.pid, signal.SIGKILL)
        process.wait(timeout=10)


def evidence_from_log(log):
    match = re.search(r'^Evidence: (.+)$', log.read_text(errors='replace'), re.MULTILINE)
    if not match:
        raise ValueError('Rendered test did not report an evidence folder')
    folder = Path(match.group(1).strip()).resolve()
    if not folder.is_relative_to(EVIDENCE.resolve()):
        raise ValueError('Test evidence is outside private/coop')
    return folder


def result_from_log(log):
    folder = evidence_from_log(log)
    result = json.loads((folder / 'result.json').read_text())
    if result.get('passed') is not True or not re.fullmatch(r'[0-9a-f]{64}', result.get('build', '')):
        raise ValueError('Rendered test has no valid passing result/build identity')
    fields = ('matching_sent_and_restored_states', 'matching_shared_world_states', 'matching_full_school_frames',
              'contact_distance', 'stationary_partner_unchanged',
              'both_models_and_particles_visible', 'checkpoint_sha256', 'resume_point', 'lifts')
    metrics = {key: result['evidence'][key] for key in fields if key in result['evidence']}
    rejoin = result['evidence'].get('rejoin')
    if rejoin:
        metrics['rejoin_preserved_resources_and_scenes'] = rejoin['resources_and_completed_scenes_preserved']
    return dict(build=result['build'], evidence=str(folder), scope=result['scope'],
                first=result.get('first'), metrics=metrics)


def failure_from_log(log, rendered):
    """Keep enough terminal evidence to triage without opening images/history."""
    result = {'log_tail': '\n'.join(log.read_text(errors='replace').splitlines()[-18:])[-6000:]}
    if not rendered:
        return result
    try:
        folder = evidence_from_log(log)
        result['evidence'] = str(folder)
        reports = json.loads((folder / 'last-reports.json').read_text())
        result['clients'] = {}
        for role in ('host', 'guest'):
            if role not in reports:
                continue
            report = reports[role]
            route = report.get('route_driver', {})
            state = dict(visit=report.get('visit'), tick=report['frame']['tick'],
                         route={k: route.get(k) for k in
                                ('command', 'phase', 'reached', 'error', 'scene', 'replans')},
                         players=[dict(feet=b['player']['feet'], grounded=b['player']['grounded'],
                                       health=b['stats']['sanity'], spawned_at=b['spawned_at'])
                                  for b in report['frame']['players']])
            command = folder / role / 'route-command.json'
            if command.exists():
                state['last_command'] = json.loads(command.read_text())
            result['clients'][role] = state
        observations = folder / 'observations.json'
        if observations.exists():
            data = json.loads(observations.read_text())
            result['last_stops'] = {k: data[k][-1] for k in
                                   ('waiting_spots', 'paired_endpoints', 'passage_stops') if data.get(k)}
    except (OSError, ValueError, KeyError, TypeError) as error:
        # A crash or timeout may leave no final report. Preserve the test's
        # original failure and log instead of making missing evidence a pass.
        result['diagnostic_error'] = str(error)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--exe', type=Path, default=ROOT / 'target/release/looking-glass.exe')
    parser.add_argument('--data', type=Path, required=True)
    parser.add_argument('--only', nargs='+', choices=('world', 'school2', 'presence', 'school',
                                                   'school-blue', 'school-red', 'lift-blue', 'lift-red',
                                                   'stairs-blue', 'stairs-red'),
                        help='school resumes a checkpoint; school-blue/red traverse the full first school; lift/stairs-blue/red are staged component checks')
    parser.add_argument('--local', action='store_true')
    parser.add_argument('--delay-ms', type=int, default=40)
    parser.add_argument('--jitter-ms', type=int, default=20)
    parser.add_argument('--school-checkpoint', type=Path)
    parser.add_argument('--school-resume-point', choices=('theatre', 'balcony', 'spiral', 'library', 'bridge', 'descent'))
    parser.add_argument('--first', choices=('blue', 'red'), default='blue',
                        help='Arrival order for the resumed school case; full cases name their own order')
    args = parser.parse_args()
    args.exe, args.data = args.exe.resolve(), args.data.resolve()
    if not args.exe.is_file() or not args.data.is_dir():
        parser.error('--exe must be a built executable and --data an existing game-data directory')
    if not (0 <= args.delay_ms <= 500 and 0 <= args.jitter_ms <= 500):
        parser.error('Delay/jitter must be between 0 and 500 ms')
    if bool(args.school_checkpoint) != bool(args.school_resume_point):
        parser.error('Provide both --school-checkpoint and --school-resume-point')
    if args.school_checkpoint:
        args.school_checkpoint = args.school_checkpoint.resolve()
        if not args.school_checkpoint.is_file():
            parser.error('The unchanged ordinary co-op checkpoint must exist')
    selected = list(dict.fromkeys(args.only or ['world', 'school2', 'presence'] +
                                 (['school'] if args.school_checkpoint else [])))
    if 'school' in selected and not args.school_checkpoint:
        parser.error('The school regression needs --school-checkpoint and --school-resume-point')
    common = ['--exe', str(args.exe), '--data', str(args.data),
              '--delay-ms', str(args.delay_ms), '--jitter-ms', str(args.jitter_ms)]
    if args.local:
        common.append('--local')
    cases = {
        'world': ([str(args.exe), '--data', str(args.data), '--coop-world-check'], 180, False),
        'school2': ([str(args.exe), '--data', str(args.data), '--school2-check'], 180, False),
        'presence': ([sys.executable, str(ROOT / 'tools/test_coop_presence.py'), *common], 300, True),
    }
    for first in ('blue', 'red'):
        cases['school-' + first] = ([sys.executable, str(ROOT / 'tools/test_coop_school.py'), *common,
                                    '--first', first, '--rejoin'], 1500, True)
        cases['lift-' + first] = ([sys.executable, str(ROOT / 'tools/test_coop_school_lift.py'), *common,
                                  '--first', first], 300, True)
        cases['stairs-' + first] = ([sys.executable, str(ROOT / 'tools/test_coop_school_stairs.py'), *common,
                                    '--first', first], 300, True)
    if args.school_checkpoint:
        cases['school'] = ([sys.executable, str(ROOT / 'tools/test_coop_school_resume.py'), *common,
                           '--checkpoint', str(args.school_checkpoint), '--resume-point', args.school_resume_point,
                           '--first', args.first, '--rejoin'], 1200, True)
    folder = EVIDENCE / ('regression-suite-' + uuid.uuid4().hex)
    folder.mkdir(parents=True)
    report_path = folder / 'summary.json'
    report = dict(format=1, status='running', passed=None, scope='Focused regressions; not full campaign certification',
                  executable=str(args.exe), executable_sha256=digest(args.exe),
                  internet=not args.local, delay_ms=args.delay_ms, jitter_ms=args.jitter_ms,
                  selected=selected, fixtures={p.name: digest(p) for p in sorted((ROOT / 'tools').glob('test_coop_*.py'))},
                  cases=[])

    def save():
        temporary = folder / 'summary.tmp'
        temporary.write_text(json.dumps(report, indent=2))
        temporary.replace(report_path)

    save()
    print('Report:', report_path, flush=True)
    interrupted = False
    build = None
    for name in selected:
        command, timeout, rendered = cases[name]
        log = folder / (name + '.log')
        case = dict(name=name, passed=False, log=str(log), timeout_seconds=timeout)
        started = time.monotonic()
        print('RUN', name, flush=True)
        try:
            with log.open('w') as output:
                process = subprocess.Popen(command, cwd=ROOT, stdout=output, stderr=subprocess.STDOUT,
                    creationflags=subprocess.CREATE_NO_WINDOW if os.name == 'nt' else 0,
                    start_new_session=os.name != 'nt')
                try:
                    case['exit_code'] = process.wait(timeout=timeout)
                except (subprocess.TimeoutExpired, KeyboardInterrupt) as error:
                    interrupted = isinstance(error, KeyboardInterrupt)
                    stop_owned_process(process)
                    raise RuntimeError('Interrupted' if interrupted else 'Test exceeded its time limit') from error
            if case['exit_code'] != 0:
                raise RuntimeError('Test failed; see its private log')
            if rendered:
                case.update(result_from_log(log))
                if build is not None and build != case['build']:
                    raise ValueError('Rendered tests used different game builds')
                build = case['build']
            # Detect accidental replacement of the executable during a suite.
            if digest(args.exe) != report['executable_sha256']:
                raise ValueError('The executable changed during the suite')
            case['passed'] = True
        except (OSError, ValueError, RuntimeError, KeyError, subprocess.SubprocessError) as error:
            case['error'] = str(error)
            if log.exists():
                case['failure'] = failure_from_log(log, rendered)
        case['seconds'] = round(time.monotonic() - started, 2)
        report['cases'].append(case)
        save()
        print('PASS' if case['passed'] else 'FAIL', name, f"({case['seconds']}s)", flush=True)
        if not case['passed']:
            break
    report.update(status='interrupted' if interrupted else 'complete',
                  passed=len(report['cases']) == len(selected) and all(c['passed'] for c in report['cases']),
                  build=build)
    save()
    print('PASS suite' if report['passed'] else 'FAIL suite', '-', report_path, flush=True)
    return 130 if interrupted else 0 if report['passed'] else 1


if __name__ == '__main__':
    raise SystemExit(main())
