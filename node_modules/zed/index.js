var LazyPromise = require('lazypromise')

module.exports = function zed() {
  var promise = arguments[0]
  var decorators = Array.prototype.slice.call(arguments, 1)

  return LazyPromise(function (resolve, reject) {
    decorators.reduce(function (promise, fn) {
      return promise.then(fn)
    }, promise).then(resolve, reject)
  })
}