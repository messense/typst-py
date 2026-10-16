import signal
import subprocess
import sys
import time

CODE = "import typst, sys; typst.Compiler(sys.argv[1]).watch(sys.argv[2])"


def wait_for(predicate, timeout=30):
    deadline = time.time() + timeout
    while time.time() < deadline:
        if predicate():
            return True
        time.sleep(0.05)
    return False


def test_watch_recompiles_on_change(tmp_path):
    main = tmp_path / "main.typ"
    lib = tmp_path / "lib.typ"
    out = tmp_path / "out.svg"
    lib.write_text("#let word = [AAA]")
    main.write_text('#import "lib.typ": word\n#word')

    proc = subprocess.Popen(
        [sys.executable, "-c", CODE, str(main), str(out)],
        stderr=subprocess.PIPE,
    )
    try:
        assert wait_for(out.exists)
        first = out.read_bytes()

        # a change to an imported file triggers a rebuild
        lib.write_text("#let word = [BBBBBBBB]")
        assert wait_for(lambda: out.read_bytes() != first)
        second = out.read_bytes()

        # a broken edit keeps the old output and watching continues
        main.write_text("#unknown-function()")
        time.sleep(1)
        assert out.read_bytes() == second
        main.write_text('#import "lib.typ": word\n#word #word')
        assert wait_for(lambda: out.read_bytes() != second)

        proc.send_signal(signal.SIGINT)
        assert proc.wait(timeout=10) != 0
        assert b"KeyboardInterrupt" in proc.stderr.read()
    finally:
        if proc.poll() is None:
            proc.kill()
        proc.stderr.close()
