#!/usr/bin/env python
# -*- coding: utf-8 -*-
from __future__ import unicode_literals
from __future__ import print_function

import os
import sys
import time
import json
import tarfile
import tempfile
import argparse
import subprocess

try:
    from urllib.request import urlopen
except ImportError:
    from urllib2 import urlopen


BASE_DIR = os.path.dirname(os.path.realpath(__file__))
REPO = 'jaemk/transfer'


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


class Download(object):
    def __init__(self, url, show_progress=True):
        self.url = url
        self.show_progress = show_progress
        self.file_size = 0
        self.bytes_read = 0

    def _dl_progress_bar(self):
        """
        Display download progress bar. If no file_size is specified. default to 100%
        """
        if not self.show_progress:
            return

        if self.file_size:
            ratio = float(self.bytes_read) / self.file_size
        else:
            ratio = 1
        percent = int(ratio * 100)

        bar_len = 60
        done = int(bar_len * ratio)
        bar = ('=' * done) + (' ' * (bar_len - done))

        progress = '{percent: >3}%: [{bar}]'.format(percent=percent, bar=bar)
        backspace = '\b' * len(progress)
        print(backspace + '\r', end='')
        print(progress, end='')

    @staticmethod
    def _get_content_length(headers):
        """
        python2 names headers are lowercase, python3 has them title-case
        """
        ctl = 'content-length'
        for k, v in headers.items():
            if k.lower() == ctl:
                return int(v)
        return None

    def to_file(self, filename):
        """
        Download the given url to specified filename
        """
        resp = urlopen(self.url)
        self.file_size = self._get_content_length(resp.headers)
        block_size = 8192
        self.bytes_read = 0
        with open(filename, 'wb') as f:
            while True:
                buf = resp.read(block_size)
                if not buf:
                    break
                self.bytes_read += len(buf)
                f.write(buf)
                self._dl_progress_bar()
        if self.show_progress:
            print(' ✓')


def get_input(prompt):
    """
    Get user input 2/3 agnostic
    """
    try:
        try:
            return raw_input(prompt)
        except NameError:
            return input(prompt)
    except EOFError:
        return ''


def confirm(msg):
    resp = get_input(msg).strip().lower()
    if resp and resp != 'y':
        return False
    return True


class Releases(object):
    @staticmethod
    def _fetch(url):
        resp = urlopen(url)
        return json.loads(resp.read().decode('utf-8'))

    @staticmethod
    def recent():
        return Releases._fetch("https://api.github.com/repos/{}/releases".format(REPO))

    @staticmethod
    def latest():
        return Releases._fetch("https://api.github.com/repos/{}/releases/latest".format(REPO))

    @staticmethod
    def tag(tag):
        return Releases._fetch("https://api.github.com/repos/{}/releases/tags/{}".format(REPO, tag))


def user_select(opts, display):
    size = len(opts)
    while True:
        for i, item in enumerate(opts):
            print("  {}: {}".format(i+1, display(item)))
        n = get_input("\nEnter the key of the selected item >> ")
        try:
            n = int(n)
            if 0 < n <= size:
                break
            else:
                print("\nError: Key `{}` out of range `{}-{}`".format(n, 1, size))
        except ValueError:
            print("\nError: Please enter a number")
    return opts[n-1]


def fetch_latest():
    latest = Releases.latest()
    tag = latest['tag_name']
    # get info on files available for download in the latest release
    bins_info = latest['assets']
    bins = [{'name': b['name'],
             'download': b['browser_download_url']} for b in bins_info]

    # # determine the target-triple to download.
    print("Please selected the which release target to download")
    selected = user_select(bins, lambda bin_: bin_['name'])

    # ex. transfer-v0.2.4-x86_64-unknown-linux-gnu.tar.gz
    target = selected['name'].rstrip('.tar.gz').split('-')[2:]
    target = '-'.join(target)

    print("The following release will be downloaded: {}".format(selected['name']))
    confirm = get_input("Do you want to continue? [Y/n] ")
    if confirm and confirm.strip().lower() != 'y':
        print("Exiting...")
        return

    # download binary tarball
    print("\n** fetching `{}`".format(selected['name']))
    Download(selected['download']).to_file(selected['name'])

    # extract binary
    print("\n** Extracting binary to `{}`".format(tag))
    tempdir = tempfile.mkdtemp()
    tar = tarfile.open(selected['name'], 'r:gz')
    tar.extractall(tempdir)
    tar.close()
    cmd('cp', '-r', os.path.join(tempdir, 'transfer'), tag)
    cmd('rm', '-rf', tempdir)

    # delete tarball
    print("** cleaning up `{}`".format(selected['name']))
    cmd('rm', '-rf', selected['name'])

    # symlink to `latest`
    confirm = get_input("\nWould you like to add/update the symlink `latest`->`{}`? [Y/n] ".format(tag))
    if confirm and confirm.strip().lower() != 'y':
        print("Exiting...")
        return
    cmd('ln', '-sfn', tag, 'latest')


def run(args):
    if args.command == 'fetch':
        fetch_latest()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(
        formatter_class=argparse.RawTextHelpFormatter,
        description=
'''
James K. <james@kominick.com>
Release update utility
Fetches latest project release from GitHub releases.
'''
        )
    subparsers = parser.add_subparsers(dest='command')
    # list_parser = subparsers.add_parser('list')
    fetch_parser = subparsers.add_parser('fetch')
    # fetch_parser.add_argument(
    #         dest='tag',
    #         type=str,
    #         help='Specify the tag of the release to download',
    #     )
    args = parser.parse_args()
    try:
        run(args)
    except CmdError as e:
        print("Error executing command: `{}`".format(e.cmd_s))
        sys.exit(e.return_code)

