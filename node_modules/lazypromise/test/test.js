var chai = require('chai')
chai.should()
var sinon = require('sinon')
chai.use(require('sinon-chai'))

var lazyPromise = require('../index')

describe('lazyPromise', function () {
  it('returns a thenable', function () {
    var p = lazyPromise(function (resolve, reject){
    })

    p.should.have.property('then')
    p.then.should.be.a('function')
  })
  it('resolves like a normal promise', function (done) {
    // TODO: integrate Promises/A+ test suite
    var p = lazyPromise(function (resolve) {
      resolve('foo')
    })

    p.then(function (val) {
      val.should.equal('foo')
    }).then(done, done)
  })
  it('doesnt call the resolver if then isnt invoked', function (done) {
    var resolver = sinon.spy()
    var p = lazyPromise(resolver)
    process.nextTick(function () {
      resolver.should.not.have.been.called
      done()
    })
  })
  it('calls the resolver after then is invoked', function (done) {
    var invoked = []
    var p = lazyPromise(function (resolve) {
      invoked.push('resolver')
      resolve()
    })

    process.nextTick(function () {
      invoked.indexOf('resolver').should.equal(-1)
      p.then(function () {
        invoked.push('fulfilled handler')
        invoked.indexOf('resolver').should.be.lessThan(invoked.indexOf('fulfilled handler'))
        done()
      })
      invoked.indexOf('resolver').should.equal(0)
    })
  })
  it('calls the resolver once', function (done) {
    var resolver = sinon.spy()
    var p = lazyPromise(resolver)
    p.then(function () {})
    p.then(function () {})
    p.then(function () {})
    process.nextTick(function () {
      resolver.should.have.been.called.once
      done()
    })
  })
})