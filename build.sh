# Prep
mkdir -p bin

# Build self
cargo build
if [ -e target/debug/pup ]; then
  cp target/debug/pup bin
fi
if [ -e target/debug/pup.exe ]; then
  cp -f target/debug/pup.exe bin
fi
# Build children
cd workers
HERE=`pwd`
for d in *; do
  echo "Build: $d"
  cd $d
  cargo build
  if [ -e target/debug/$d.exe ]; then
    cp target/debug/$d.exe $HERE/../bin
  fi
  if [ -e target/debug/$d ]; then
    cp target/debug/$d $HERE/../bin
  fi
  cd $HERE
done
