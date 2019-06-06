# Vendor self
mkdir -p .cargo
cargo vendor > .cargo/vendor

# Vendor children
cd workers
HERE=`pwd`
for d in *; do
  echo "Vendor: $d"
  cd $d
  mkdir -p .cargo
  cargo vendor > .cargo/vendor
  cd $HERE
done
