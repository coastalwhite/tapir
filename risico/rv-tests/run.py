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

log_level = LogLevel.WARNING
log_output = dict(
    error = sys.stderr,
    warning = sys.stderr,
    info = sys.stdout,
)

def log_error(s: str):
    if log_level.value >= LogLevel.ERROR.value:
        print('[ERROR]: ' + s, file=log_output['info'])

def log_warning(s: str):
    if log_level.value >= LogLevel.WARNING.value:
        print('[WARN]: ' + s, file=log_output['info'])

def log_info(s: str):
    if log_level.value >= LogLevel.INFO.value:
        print('[INFO]: ' + s, file=log_output['info'])

class Settings:
    march: str = 'i'
    features: list[str] = []

    def set_option(self, option: str, value: str | None) -> bool:
        match option:
            case 'march':
                assert value != None
                self.march = value
                return True
            case 'features':
                assert value != None
                self.features = list(
                    map(lambda f: f.strip().lower(), value.split())
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

            split = line.split(SETTING_VALUE_DELIM, 1)

            option = split[0].strip()
            value = None if len(split) == 1 else split[1].strip()

            if not settings.set_option(option, value):
                raise Exception(f"{path}:{i}: Unknown option '{option}'")

    return settings


def compile_rs(path: str, out: str):
    settings = get_file_settings(path, RUST_COMMENT)

    target = f"riscv32{settings.march}-unknown-none-elf"
    features = settings.features

    argv = ["rustc", path, "--target", target, "-C", "panic=abort", "-o", out]

    if len(features) != 0:
        feature_list = map(lambda f: '+' + f, features)
        argv += ["-C", "target-feature=" + ','.join(feature_list)]

    log_info('cmd = ' + ' '.join(argv))

    rustc = subprocess.run(argv, capture_output = True)
    rustc.check_returncode()

def compile_asm(path: str, out: str):
    settings = get_file_settings(path, ASM_COMMENT)

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

def run_elf(root: str, path: str):
    cli_bin = os.path.join(root, 'target', 'debug', 'cli')
    argv = [cli_bin, "-S", "testing", path]

    log_info('cmd = ' + ' '.join(argv))
    risico = subprocess.run(argv, capture_output = True)
    risico.check_returncode()

def run_bin(root: str, path: str):
    cli_bin = os.path.join(root, 'target', 'debug', 'cli')
    argv = [cli_bin, "-R", "-S", "testing", path]

    log_info('cmd = ' + ' '.join(argv))
    risico = subprocess.run(argv, capture_output = True)
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


def find_project_dirs() -> dict[str, str]:
    TESTS_ROOT = os.path.dirname(os.path.realpath(__file__))
    PROJECT_ROOT = os.path.dirname(TESTS_ROOT)

    return dict(
        TESTS_ROOT = TESTS_ROOT,
        PROJECT_ROOT = PROJECT_ROOT,
        OUTPUT = os.path.join(TESTS_ROOT, 'build'),
    )

def main():
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
    except Exception as e:
        log_error(f'Failed to compile risico. Reason: {e}')
        eprint(traceback.format_exc())

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
            bin = os.path.join(project_dirs['OUTPUT'], name, t)
            src = t + '.' + ext

            try:
                os.makedirs(os.path.dirname(bin))
            except:
                pass

            try:
                cat_compile(src, bin)
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

    failed_runs = 0
    for name, [tests, _, _, run] in categories.items():
        print()
        print(f"[{name}]:")
        for t in tests:
            bin = os.path.join(project_dirs['OUTPUT'], name, t)

            try:
                run(project_dirs['PROJECT_ROOT'], bin)
                print(f"{t}: success")
            except subprocess.CalledProcessError as e:
                failed_runs += 1

                eprint(f"{t}: error. return code = {e.returncode}")
                eprint('--- STDERR ---')
                eprint(e.stderr.decode(), end='')
                eprint('--------------')
            except Exception as e:
                failed_runs += 1

                eprint(f"{t}: error. reason = {e}")
                eprint(traceback.format_exc())

    if failed_runs > 0:
        eprint()
        total_tests = sum(map(lambda c: len(c[0]), categories.values()))
        log_error(f'error: {failed_runs}/{total_tests} failed...')
        exit(1)

if __name__ == '__main__':
    main()