FROM solanalabs/solana:v1.7.4

RUN [ "apt", "upgrade" ]
RUN [ "apt", "update" ]
RUN [ "apt", "install", "curl", "-y" ]
RUN [ "apt" , "install", "build-essential", "-y"]
RUN [ "apt" , "install", "cmake", "-y"]

RUN curl https://sh.rustup.rs -sSf | bash -s -- -y
RUN echo 'source $HOME/.cargo/env' >> $HOME/.bashrc

COPY . /app
WORKDIR /app

ENTRYPOINT [ "bash" ]
