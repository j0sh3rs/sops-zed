var chai = require('chai')
chai.should()
var Q = require('q')
var isPromise = require('is-promise')
var sinon = require('sinon')
chai.use(require('sinon-chai'))

var zed = require('../index')


describe('zed', function () {
  
  it('returns a promise', function () {

    var promise = Q.resolve('foo')

    var z = zed(promise)
    isPromise(z).should.equal(true)

  })


  it('composes promise decorators', function (done) {

    var promise = Q.resolve('asdf')
    var a = sinon.stub().returns('A')
    var b = sinon.stub().returns('B')
    var c = sinon.stub().returns('C')

    var z = zed(promise,
            a,
            b,
            c)

    z.then(function (val) {
      a.should.have.been.calledWithExactly('asdf')
      b.should.have.been.calledAfter(a)
      b.should.have.been.calledWithExactly('A')
      c.should.have.been.calledAfter(b)
      c.should.have.been.calledWithExactly('B')
      val.should.equal('C')

    }).then(done, done)

  })

  it('attaches then onFulfilled handlers lazily', function (done) {
    var thenable = {then: sinon.stub().returns(Q.resolve('thenable'))}
    var I = function (x) { return x }

    var z = zed(thenable, I, I, I)

    process.nextTick(function () {
      thenable.then.should.not.have.been.called

      z.then(I)
      z.then(I)
      z.then(function (val) {
        val.should.equal('thenable')
        thenable.then.should.have.been.calledOnce
      }).then(done, done)

    })

  })

})