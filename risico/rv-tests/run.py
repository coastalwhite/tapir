#!/usr/bin/env python

import sys
import os
import shutil
import subprocess
import traceback
from enum import Enum

RUST_COMMENT = '//'
ASM_COMMENT = '#'

SETTING_START = 'option:'
SETTING_VALUE_DELIM = '='

ERROR_NUM_LINES = 50
TIMEOUT = 3

EXCLUDES = [
    'common',
    'build',
    'old',
]

def eprint(*args, **kwargs):
    print(*args, file=sys.stderr, **kwargs)

class LogLevel(Enum):
    NONE = 0
    ERROR = 1
    WARNING = 2
    INFO = 3

LOG_LEVEL = LogLevel.WARNING
LOG_OUTPUT = dict(
    error = sys.stderr,
    warning = sys.stderr,
    info = sys.stdout,
)

def log_error(s: str):
    global LOG_LEVEL
    if LOG_LEVEL.value >= LogLevel.ERROR.value:
        print('[ERROR]: ' + s, file=LOG_OUTPUT['error'])

def log_warning(s: str):
    global LOG_LEVEL
    if LOG_LEVEL.value >= LogLevel.WARNING.value:
        print('[WARN]: ' + s, file=LOG_OUTPUT['warning'])

def log_info(s: str):
    global LOG_LEVEL
    if LOG_LEVEL.value >= LogLevel.INFO.value:
        print('[INFO]: ' + s, file=LOG_OUTPUT['info'])

def display_error_truncated(s: str):
    lines = s.splitlines()
    if len(lines) > ERROR_NUM_LINES:
        truncated_lines = len(lines) - ERROR_NUM_LINES
        displayed_lines = lines[-ERROR_NUM_LINES:]
        eprint(f'({truncated_lines} hidden lines)\n' + '\n'.join(displayed_lines) + '\n', end='')
    else:
        eprint(s, end='')

class Settings:
    march: str = 'i'
    features: list[str] = []
    allow_trap: list[str] = []

    def set_option(self, option: str, value: str | None) -> bool:
        match option:
            case 'march':
                assert type(value) is str
                self.march = value
                return True
            case 'features':
                assert type(value) is str
                self.features = list(
                    map(lambda f: f.strip().lower(), value.split(','))
                )
                return True
            case 'allow_trap':
                if type(value) is bool:
                    self.allow_trap = ['all'] if value else []
                elif type(value) is str:
                    self.allow_trap = list(
                        map(lambda f: f.strip().lower(), value.split(','))
                    )

                return True

        return False

def compile_risico(root: str):
    argv = [f"cargo", "build", "--bin", "cli"]

    log_info('cwd = ' + root + ', cmd = ' + ' '.join(argv))

    cargo = subprocess.run(argv, cwd = root, capture_output = True)
    cargo.check_returncode()

def get_file_settings(path: str, comment: str) -> Settings:
    settings = Settings()

    with open(path, 'r') as content:
        lines = content.readlines()

        for i, line in enumerate(lines):
            line = line.strip()

            if not line.startswith(comment):
                continue

            line = line[len(comment):].strip()

            if not line.startswith(SETTING_START):
                continue

            line = line[len(SETTING_START):].strip()

            if '=' not in line:
                if line.startswith('!'):
                    option = line[1:].strip()
                    value = False
                else:
                    option = line
                    value = True
            else:
                split = line.split(SETTING_VALUE_DELIM, 1)

                option = split[0].strip()
                value = None if len(split) == 1 else split[1].strip()

            if not settings.set_option(option, value):
                raise Exception(f"{path}:{i}: Unknown option '{option}'")

    return settings


def compile_rs(path: str, out: str, settings: Settings):
    target = f"riscv32{settings.march}-unknown-none-elf"
    features = settings.features

    argv = ["rustc", path, "--target", target, "-C", "panic=abort", "-o", out]

    if len(features) != 0:
        feature_list = map(lambda f: '+' + f, features)
        argv += ["-C", "target-feature=" + ','.join(feature_list)]

    log_info('cmd = ' + ' '.join(argv))

    rustc = subprocess.run(argv, capture_output = True)
    rustc.check_returncode()

def compile_asm(path: str, out: str, settings: Settings):
    march = 'rv32' + settings.march
    features = settings.features

    if len(features) != 0:
        march += ''.join(features)

    outbin = out + '.elf'

    argv = ["riscv32-none-elf-as", path, "-march", march, "-o", outbin]

    log_info('cmd = ' + ' '.join(argv))
    riscv_as = subprocess.run(argv, capture_output = True)
    riscv_as.check_returncode()

    argv = ["riscv32-none-elf-objcopy", "-O", "binary", outbin, out]

    log_info('cmd = ' + ' '.join(argv))
    riscv_objcopy = subprocess.run(argv, capture_output = True)
    riscv_objcopy.check_returncode()

def run_elf(root: str, path: str, settings: Settings, syscalls = 'testing', htif = False):
    cli_bin = os.path.join(root, 'target', 'debug', 'cli')
    argv = [cli_bin, "-S", syscalls]
    argv += ['-t', 'all']
    for v in settings.allow_trap:
        argv += ['-t', v + '=allow']
    if htif:
        argv += ['--htif']
    argv += [path]

    log_info('cmd = ' + ' '.join(argv))
    risico = subprocess.run(argv, capture_output = True, timeout = TIMEOUT)
    risico.check_returncode()

def run_bin(root: str, path: str, settings: Settings):
    cli_bin = os.path.join(root, 'target', 'debug', 'cli')
    argv = [cli_bin, "-R", "-S", "testing"]
    argv += ['-t', 'all']
    for v in settings.allow_trap:
        argv += ['-t', v + '=allow']
    argv += [path]

    log_info('cmd = ' + ' '.join(argv))
    risico = subprocess.run(argv, capture_output = True, timeout = TIMEOUT)
    risico.check_returncode()

def find_all_with_ext(root: str, ext: str) -> list[str]:
    output = []

    for (walkroot,_,files) in os.walk('.', topdown=True):
        for file in files:
            path = os.path.join(walkroot, file)
            path = os.path.relpath(path, root)

            if not file.endswith('.' + ext):
                continue

            do_exclude = False
            for exclude in EXCLUDES:
                if path.startswith(exclude):
                    do_exclude = True
                    break

            if do_exclude:
                continue

            output.append(path)

    return output

def find_all_unit_tests(
    root: str,
    category_whitelist: list[str],
    tvm_whitelist: list[str]
) -> list[tuple[str, str]]:
    output = []

    for (walkroot,_,files) in os.walk(root, topdown=True):
        for file in files:
            path = os.path.join(walkroot, file)

            if not file.startswith('rv'):
                continue

            if file.endswith('.dump'):
                continue

            [category, tvm, name] = file.split('-', maxsplit=2)

            if tvm not in tvm_whitelist:
                continue

            if category not in category_whitelist:
                continue

            output.append((file, path))

    return output

def find_project_dirs() -> dict[str, str]:
    TESTS_ROOT = os.path.dirname(os.path.realpath(__file__))
    PROJECT_ROOT = os.path.dirname(TESTS_ROOT)
    RISCV_TESTS = os.getenv("RISCV_TESTS")

    if RISCV_TESTS == None:
        eprint('The environment variable $RISCV_TESTS is not set. Make sure to run within the devShell with `nix develop`')
        exit(1)

    return dict(
        TESTS_ROOT = TESTS_ROOT,
        PROJECT_ROOT = PROJECT_ROOT,
        RISCV_TESTS = RISCV_TESTS,
        OUTPUT = os.path.join(TESTS_ROOT, 'build'),
    )

def main():
    global LOG_LEVEL
    argv = sys.argv

    args = argv[1:]

    set_log_flags = [s.split('=', 1)[1] for s in args if s.startswith('--log=') ]
    if len(set_log_flags) != 0:
        log_level = set_log_flags[-1].strip()
        match log_level:
            case 'none': LOG_LEVEL = LogLevel.NONE
            case 'error': LOG_LEVEL = LogLevel.ERROR
            case 'warning': LOG_LEVEl = LogLevel.WARNING
            case 'info': LOG_LEVEL = LogLevel.INFO
            case _:
                eprint(f"Invalid Log Level {log_level}!")
                exit(2)
        log_info(f"LOG_LEVEL = {log_level}")

    FILTERS = [f for f in args if not f.startswith('-')]
    if len(FILTERS) == 0:
        FILTERS = None

    log_info(f"FILTERS = {FILTERS}")

    project_dirs = find_project_dirs()

    searches = {
        'rs': { 'ext': 'rs', 'compile': compile_rs, 'run': run_elf },
        'asm': { 'ext': 'S', 'compile': compile_asm, 'run': run_bin },
    }

    categories = {}
    for (name, search) in searches.items():
        files = find_all_with_ext(project_dirs['TESTS_ROOT'], search['ext'])
        tests = list(map(lambda f: f[:-(1 + len(search['ext']))], files))

        categories[name] = [
            tests,
            search['ext'],
            search['compile'],
            search['run'],
        ]

    try:
        compile_risico(project_dirs['PROJECT_ROOT'])
    except subprocess.CalledProcessError as e:
        log_error(f'Failed to compile risico')
        eprint(e.stderr.decode())
        eprint()
        exit(1)
    except Exception as e:
        log_error(f'Failed to compile risico. Reason: {e}')
        eprint(traceback.format_exc())
        exit(1)

    try:
        shutil.rmtree(project_dirs['OUTPUT'])
    except:
        pass

    try:
        os.makedirs(project_dirs['OUTPUT'])
    except:
        pass

    keep = os.path.join(project_dirs['OUTPUT'], '.gitkeep')
    with open(keep, 'a'):
        os.utime(keep, None)

    failed_compiles = 0
    for name, [tests, ext, cat_compile, _] in categories.items():
        for t in tests:
            if FILTERS != None and not any([f in t for f in FILTERS]):
                continue

            bin = os.path.join(project_dirs['OUTPUT'], name, t)
            src = t + '.' + ext

            try:
                os.makedirs(os.path.dirname(bin))
            except:
                pass

            try:
                settings = get_file_settings(src, RUST_COMMENT if ext == 'rs' else ASM_COMMENT)
                cat_compile(src, bin, settings)
            except subprocess.CalledProcessError as e:
                failed_compiles += 1

                log_error(f'Failed to compile {src}')
                eprint(e.stderr.decode())
                eprint()
            except Exception as e:
                failed_compiles += 1

                log_error(f'Failed to compile {src}. Reason: {e}')
                eprint(traceback.format_exc())

    if failed_compiles > 0:
        eprint()
        log_error(f'Failed to compile {failed_compiles} tests...')
        exit(1)

    num_tests = 0
    num_ignored = 0
    failed_runs = 0
    for name, [tests, ext, _, run] in categories.items():
        print()
        print(f"[{name}]:")
        for t in tests:
            if FILTERS != None and not any([f in t for f in FILTERS]):
                continue

            num_tests += 1

            src = t + '.' + ext
            bin = os.path.join(project_dirs['OUTPUT'], name, t)

            try:
                print(f"{t}: \r", end = '')
                settings = get_file_settings(src, RUST_COMMENT if ext == 'rs' else ASM_COMMENT)
                run(project_dirs['PROJECT_ROOT'], bin, settings)
                print(f"{t}: success")
            except subprocess.CalledProcessError as e:
                failed_runs += 1

                eprint(f"{t}: error. return code = {e.returncode}")
                eprint('--- STDERR ---')
                display_error_truncated(e.stderr.decode())
                eprint('--------------')
            except subprocess.TimeoutExpired as e:
                failed_runs += 1

                eprint(f"{t}: timeout.")
                eprint('--- STDERR ---')
                display_error_truncated(e.stderr.decode())
                eprint('--------------')
            except Exception as e:
                failed_runs += 1

                eprint(f"{t}: error. reason = {e}")
                eprint(traceback.format_exc())

    CATEGORY_WHITELIST = [
        'rv32mi',
        'rv32ui',
        'rv32um',
        'rv32uc',
        'rv32uf',
    ]
    TVM_WHITELIST = [
        'p',
    ]

    IGNORED = [
        'rv32mi-p-breakpoint', # This assumes the presence of the RISC-V Debug Standard
    ]

    print()
    print(f"[riscv-tests unit tests]:")
    unit_tests_path = os.path.join(project_dirs['RISCV_TESTS'], 'share', 'riscv-tests', 'isa')
    unit_tests = find_all_unit_tests(unit_tests_path, CATEGORY_WHITELIST, TVM_WHITELIST)
    for (file, path) in unit_tests:
        if FILTERS != None and not any([f in file for f in FILTERS]):
            continue

        num_tests += 1

        if file in IGNORED:
            num_ignored += 1
            print(f"{file}: ignored")
            continue

        try:
            print(f"{file}: \r", end = '')

            settings = Settings()
            settings.allow_trap = ['all']

            category = file.split('-')[0]

            run_elf(project_dirs['PROJECT_ROOT'], path, settings, syscalls='trapvec', htif = True)
            print(f"{file}: success")
        except subprocess.CalledProcessError as e:
            failed_runs += 1

            eprint(f"{file}: error. return code = {e.returncode}")
            eprint('--- STDERR ---')
            display_error_truncated(e.stderr.decode())

            eprint('--------------')
        except subprocess.TimeoutExpired as e:
            failed_runs += 1

            eprint(f"{file}: timeout.")
            eprint('--- STDERR ---')
            display_error_truncated(e.stderr.decode())
            eprint('--------------')
        except Exception as e:
            failed_runs += 1

            eprint(f"{file}: error. reason = {e}")
            eprint(traceback.format_exc())

    if failed_runs > 0:
        eprint()
        log_error(f'error: {failed_runs}/{num_tests} failed...')
        log_error(f'ignored: {num_ignored}/{num_tests}...')
        exit(1)

if __name__ == '__main__':
    main()