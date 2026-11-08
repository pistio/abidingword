"""Runs one command from a fresh, small process and prints its wall time,
CPU time and peak resident memory as JSON (os.wait4). A child forked from a
large process starts with that process's resident size, which inflates a
small child's peak (A2.1: the measuring process holds the reference texts),
so each reading is started from here instead.

usage: python3 -I clean_run.py <timeout seconds> <command> [args...]
       (stdout of the command is discarded; environment passed through)
"""
import json, os, signal, subprocess, sys, threading, time

def main():
    timeout = float(sys.argv[1]); cmd = sys.argv[2:]
    t0 = time.perf_counter()
    p = subprocess.Popen(cmd, stdout=subprocess.DEVNULL, stderr=subprocess.PIPE,
                         cwd=os.environ.get('AW_CLEAN_CWD'))
    err = []
    th = threading.Thread(target=lambda: err.append(p.stderr.read())); th.start()
    killed = [False]
    def kill():
        killed[0] = True
        try: os.kill(p.pid, signal.SIGKILL)
        except ProcessLookupError: pass
    tm = threading.Timer(timeout, kill); tm.start()
    _, status, ru = os.wait4(p.pid, 0)
    wall = time.perf_counter() - t0
    tm.cancel(); th.join()
    print(json.dumps({'rc': os.waitstatus_to_exitcode(status), 'seconds': round(wall, 3),
                      'cpu_seconds': round(ru.ru_utime + ru.ru_stime, 3),
                      'peak_rss_mb': round(ru.ru_maxrss / 1024, 1), 'timeout': killed[0]}))

if __name__ == '__main__':
    main()
