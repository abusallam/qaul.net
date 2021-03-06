#!/bin/sh

# build and upload the contributors manual
cd contributors
./deploy.sh
cd ..

# build and upload the HTTP-API documentation
cd http-api
./deploy.sh
cd ..

# build and upload the users manual
cd users/
./deploy.sh
cd ..

# build and upload the rust documentation
cd ..
cargo doc --no-deps
cp target/doc/settings.html target/doc/index.html
rsync -azzhe "ssh -p 2222" ./target/doc/ admin@docs.qaul.net:/home/admin/api
cd docs
