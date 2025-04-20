var Promise = require('promise')

function lazyPromise(resolver) {
  var p
  var lazy = {
    then: function () {
      p = p || new Promise(resolver)
      return p.then.apply(this, arguments)
    }
  }

  return lazy
}

module.exports = lazyPromise