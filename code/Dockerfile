FROM rust:1.64.0

RUN rustup target add aarch64-unknown-linux-gnu

RUN apt-get install gcc-aarch64-linux-gnu

RUN wget -O hwloc-2.8.0.tar.bz2 https://download.open-mpi.org/release/hwloc/v2.8/hwloc-2.8.0.tar.bz2
RUN tar -xf hwloc-2.8.0.tar.bz2
RUN rm hwloc-2.8.0.tar.bz2

WORKDIR /hwloc-2.8.0
RUN export CC="aarch64-linux-gnu-gcc"
RUN export CPP="aarch64-linux-gnu-cpp"
RUN export CXX="aarch64-linux-gnu-gcc"
RUN export CXXCPP="aarch64-linux-gnu-g++"
RUN export HOST_TRIPLE="aarch64-unknown-linux-gnu"
RUN ./configure --prefix=/home/hwloc --enable-static --build=aarch64-unknown-linux-gnu --host=aarch64-unknown-linux-gnu  --disable-libxml2
RUN make
RUN make install

WORKDIR /code

RUN cargo init
# Replacing with the desired dependencies
COPY Cargo.toml Cargo.lock DockerConfig.toml ./
# Dependencies built and cached, will be reuilt when Cargo.toml is changed
RUN cargo build --config=DockerConfig.toml

COPY . .

RUN cargo build --bin gpio --package led-matrix-controller --config=DockerConfig.toml --features="rppal scheduler"

CMD mkdir -p dockerOutput; cp target/aarch64-unknown-linux-gnu/debug/gpio dockerOutput/
