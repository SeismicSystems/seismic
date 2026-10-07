import seismic = require('seismic-encrypt')

// CommonJS consumers also resolve the public declarations.
declare const params: seismic.EncryptSeismicTxParams
const result = seismic.encryptSeismicTx(params)
void result.then(({ serialize, seismicTx }) => {
  const payment: seismic.GasPayment = seismicTx.gasPayment
  serialize({ r: '0x01', s: '0x02', yParity: 0 })
  serialize({ r: '0x01', s: '0x02', v: 27n })
  serialize()
  return payment
})
