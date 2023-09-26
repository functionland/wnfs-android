# WNFS-lib

This is the home of the _Rust_ WNFS library for Android which exposes multi-platform functions ready for being wrapped by different interfaces.

## Compile

```bash
export JAVA_HOME=path/to/java8
cd PROJECT/lib/src/main/java;
$JAVA_HOME/jre/bin/javac -cp ./ ./land/fx/wnfslib/InMemoryDatastore.java
$JAVA_HOME/jre/bin/javac -cp ./ ./land/fx/wnfslib/result/*
$JAVA_HOME/jre/bin/javac -cp ./ ./land/fx/wnfslib/exceptions/*
cd PROJECT/wnfslib;
RUST_BACKTRACE=1 LD_LIBRARY_PATH=$JAVA_HOME/lib/server cargo test
```
