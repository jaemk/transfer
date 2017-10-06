#!/usr/bin/env python

import os
import sys
import time
import argparse
import subprocess

PROJDIR = os.path.dirname(os.path.realpath(__file__))


class CmdError(Exception):
    def __init__(self, cmd_s=None, return_code=None, *args, **kwargs):
        self.cmd_s = cmd_s
        self.return_code = return_code
        msg = "Command: `{}` exited with status: {}".format(cmd_s, return_code)
        super(CmdError, self).__init__(msg, *args, **kwargs)


def cmd(*args, **kwargs):
    """
    Run `*args` in a subprocess, piping its output to stdout.
    Additional `**kwargs` are passed to `Popen`
    Subprocess errors are captured and raised as `CmdError`s
    """
    cmd_s = ' '.join(args)
    print('+ {}'.format(cmd_s))
    proc = subprocess.Popen(cmd_s, shell=True, stdout=subprocess.PIPE, **kwargs)
    for line in iter(proc.stdout.readline, ''):
        sys.stdout.write('> {}'.format(line))
    while proc.poll() is None:
        time.sleep(0.5)
    if proc.returncode != 0:
        raise CmdError(cmd_s, proc.returncode)


def run(version=None, no_migrate=False):
    proj_cmd = lambda *args: cmd(*args, cwd=PROJDIR)
    print("** Updating project files **")

    if version is not None:
        proj_cmd('git', 'fetch', '--all', '--tags')
        proj_cmd('git', 'checkout', version)
    else:
        proj_cmd('git', 'pull', '--rebase=false')

    if not no_migrate:
        proj_cmd('bin/x86_64/transfer', 'admin', 'database', 'migrate')
    proj_cmd('sudo', 'systemctl', 'restart', 'transfer')


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--version', '-v', type=str)
    parser.add_argument('--no-migrate', '-nm', action='store_true')
    args = parser.parse_args()
    try:
        v = getattr(args, 'version', None)
        run(v, args.no_migrate)
    except CmdError as e:
        print("Error executing command: `{}`".format(e.cmd_s))
        sys.exit(e.return_code)

