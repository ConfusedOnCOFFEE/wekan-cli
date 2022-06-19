# Coverage

Testing ground. No option is recommended.

## GRCOV

[GRCOV](https://github.com/mozilla/grcov)

Build: `rustup component add llvm-tools-preview`
Export: `export RUSTFLAGS="-Cinstrument-coverage"`


## KCOV

Install:

```basg
apt-get install libcurl4-openssl-dev libelf-dev libdw-dev cmake gcc
```

Build KCOV:

```bash
wget https://github.com/SimonKagstrom/kcov/archive/master.tar.gz
tar xzf master.tar.gz
cd kcov-master
mkdir build
cd build
cmake ..
make
sudo make install
```

Collect coverage:

```bash
cargo test --no-run
kcov target/cov target/debug/$TEST_EXECUTABLE
```


Thx to [lifthrasiir](https://users.rust-lang.org/u/lifthrasiir/summary)

```bash
#!/bin/bash
PKGID="$(cargo pkgid)"
[ -z "$PKGID" ] && exit 1
ORIGIN="${PKGID%#*}"
ORIGIN="${ORIGIN:7}"
PKGNAMEVER="${PKGID#*#}"
PKGNAME="${PKGNAMEVER%:*}"
shift
cargo test --no-run || exit $?
EXE=($ORIGIN/target/debug/$PKGNAME-*)
if [ ${#EXE[@]} -ne 1 ]; then
    echo 'Non-unique test file, retrying...' >2
    rm -f ${EXE[@]}
    cargo test --no-run || exit $?
fi
rm -rf $ORIGIN/target/cov
kcov $ORIGIN/target/cov $ORIGIN/target/debug/$PKGNAME-* "$@"
```

- [Reference](https://users.rust-lang.org/t/tutorial-how-to-collect-test-coverages-for-rust-project/650)
- [Github](https://github.com/mozilla/grcov)
