# This script takes care of building your crate and packaging it for release

set -ex

main() {
    local src=$(pwd) \
          stage=

    case $TRAVIS_OS_NAME in
        linux)
            stage=$(mktemp -d)
            ;;
        osx)
            stage=$(mktemp -d -t tmp)
            ;;
    esac

    test -f Cargo.lock || cargo generate-lockfile

    # Build artifacts
    ./build.py web

    cross rustc --bin transfer --target $TARGET --release -- -C lto
    mkdir -p bin
    cp target/$TARGET/release/transfer bin/

    rm -rf target/
    rm -rf web/node_modules
    rm -rf web/build

    mkdir -p $stage/transfer
    cp -r * $stage/transfer

    cd $stage
    tar czf $src/$CRATE_NAME-$TRAVIS_TAG-$TARGET.tar.gz *
    cd $src

    rm -rf $stage
}

main
