import { encryptSeismicTx, serializeSeismicTx } from 'seismic-encrypt'
import type {
  EncryptSeismicTxParams,
  GasPayment,
  SeismicSerializableTransaction,
  SeismicTxSerializer,
} from 'seismic-encrypt'
import type { Signature } from 'viem'
import { privateKeyToAccount } from 'viem/accounts'

// Compiled against public built declarations, with no workspace source aliases.
declare const params: EncryptSeismicTxParams
const account = privateKeyToAccount(
  '0x0000000000000000000000000000000000000000000000000000000000000001'
)
const { seismicTx, serialize, unsignedSerializedTx } =
  await encryptSeismicTx(params)
const resolved: GasPayment = seismicTx.gasPayment
const standalone: SeismicSerializableTransaction = seismicTx
const signed = await account.signTransaction<SeismicTxSerializer>(seismicTx, {
  serializer: serializeSeismicTx,
})
const callback: SeismicTxSerializer = (_tx, signature) => serialize(signature)
await account.signTransaction<SeismicTxSerializer>(seismicTx, {
  serializer: callback,
})
const parityOnly: Signature = { r: '0x01', s: '0x02', yParity: 0 }
serialize(parityOnly)
serialize({ r: '0x01', s: '0x02', v: 27n })
serialize()
serializeSeismicTx(standalone, parityOnly)
void [resolved, signed, unsignedSerializedTx]
