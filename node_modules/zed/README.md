# zed
lazily map functions to a lazy promise

## usage

```javascript
var zed = require('zed')

var z = zed(promise,
            a,
            b,
            c)

z.then(function (val) {
  console.log('we forced evaluation of the lazy promise, then ran it through some things!')
})
```

where `a`, `b`, and `c` are functions like you would pass as onFulfilled handlers to `promise.then()`. That is, if it throws, the resultant promise will be rejected. If it returns a value, that value will be used as the argument for the next function. If it returns a promise, the sequence will wait on that promise to be fulfilled and then use that value as the argument on the next step in the chain.

## api

### `zed: (Promise, ...Function(Value) => Promise|Value) => LazyPromise`

The first argument is any Promises/A+ compatible thenable. Most commonly, it will be a Promise or a LazyPromise. The rest of the arguments are the functions to chain in series order. The functions will not attach `.then` listeners until the resultant LazyPromise is forced, thus preserving lazy evaluation if the upstream promise is lazy.

## installation

    $ npm install zed

## running the tests

from project root:

    $ npm install
    $ npm test

## contributors

- jden <jason@denizac.org>

## license

MIT. (c) 2013 jden <jason@denizac.org>. See LICENSE.md