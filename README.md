# Reverse Proxy

### Usage

#### Start the server
In one terminal start the proxy server:
```bash
git clone git@github.com:taylorjdawson/rev-proxy.git
cd rev-proxy
cargo run
```

#### Proxy requests
Proxy api takes the origin server's domain via the `url` parameter like so:
```
/?url=https://expample.com
```

In another terminal issue a curl request through the proxy:
```
curl https://localhost:8080?url=http://worldtimeapi.org/api/ip
```

#### TLS
Project includes cert/key.pem files in order to proxy secure https requests.
You will need to install and use `mkcert` in order to use tls with localhost.

Assuming you are on MacOs:
```
brew install mkcert

// Created a new local CA
mkcert -install

// Generate cert for localhost
mkcert localhost
```
At this point you should be able to hit `https://localhost` without any cert errors.

The above is terse explanation for convenience. Should you encounter any issues please refer to the [mkcert README](https://github.com/FiloSottile/mkcert) for more details on installation and usage.
