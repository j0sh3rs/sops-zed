# lazypromise
lazy promises which start on then

## about

LazyPromise is Promises/A+ compatible. Most promise implementations begin doing the underlying work as soon as the promise is created. LazyPromise puts that off until the promised value is needed. For example, a LazyPromise for a remote resource will avoid making the underlying network request until its `.then()` method is called.

## usage
```javascript
var LazyPromise = require('lazypromise')
```

Use LazyPromise with a [`promise`](https://npmjs.org/package/promise)-style resolver function.

```javascript
var lazyPromise = LazyPromise(function (resolve, reject) {
  resolve('foo')
})
```

At this point, the resolver has not been called. `lazyPromise` is a Promises/A+ compatible promise which you can pass around your application. However, the resolver (which might be an expensive computation, database query, etc.) will not be invoked until a fulfilled or rejected handler is added using `.then()`. If nothing ever calls `.then()`, then the resolver will never be invoked!

## installation

    $ npm install lazypromise

## running the tests

from project root:

    $ npm install
    $ npm test

## contributors

- jden <jason@denizac.org>

## license

MIT. (c) 2013 jden <jason@denizac.org>. See LICENSE.md