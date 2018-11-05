# Prep
mkdir -p bin

# Build self
cargo build --release --frozen
if [ -e target/release/pup ]; then
  cp target/release/pup bin
fi
if [ -e target/release/pup.exe ]; then
  cp -f target/release/pup.exe bin
fi
# Build children
cd workers
HERE=`pwd`
for d in *; do
  echo "Build: $d"
  cd $d
  cargo build --release --frozen
  if [ -e target/release/$d.exe ]; then
    cp target/release/$d.exe $HERE/../bin
  fi
  if [ -e target/release/$d ]; then
    cp target/release/$d $HERE/../bin
  fi
  cd $HERE
done
