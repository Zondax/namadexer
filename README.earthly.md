# Building earthly containers

## Install earthly

Install earthly from [here](https://earthly.dev/get-earthly)

## Running

If you have WARP access, you can speed up your builds by defining: 

```bash
export EARTHLY_BUILDKIT_HOST=tcp://buildkit.zondax.dev:8372
earthly config global.tls_enabled false
```

To build `earthly --ci +all`
To push `earthly --ci --push +all`
