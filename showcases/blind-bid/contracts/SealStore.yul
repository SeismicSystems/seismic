// SealStore — библиотека приватного хранилища для Seismic.
//
// Обычный solc не умеет shielded-типы (suint256/sbool), но Seismic добавил опкоды
// CLOAD (0xB0) и CSTORE (0xB1) — «FlaggedStorage»: слот помечается приватным,
// и внешний наблюдатель видит в нём нули вместо значения.
//
// Поэтому библиотека написана на Yul: значения читаются/сравниваются/складываются
// внутри TEE и наружу отдаются только производные (слот победителя, сумма), но не сами суммы.
//
// Функции:
//   store(uint256 slot, uint256 value)        — записать приватное значение
//   storeFromTransient(uint256 slot)          — записать значение из transient storage (TSTORE)
//   load(uint256 slot) → uint256              — прочитать приватное значение
//   minSlot(uint256[] slots) → uint256        — слот с минимальным значением (сравнение внутри)
//   sumSlots(uint256[] slots) → uint256       — сумма значений (агрегат для аудита)

object "SealStore" {
  code {
    let size := datasize("SealStore_runtime")
    datacopy(0, dataoffset("SealStore_runtime"), size)
    return(0, size)
  }

  object "SealStore_runtime" {
    code {
      if lt(calldatasize(), 4) { revert(0, 0) }
      let sel := shr(224, calldataload(0))

      switch sel

      // store(uint256 slot, uint256 value)
      case 0x6ed28ed0 {
        verbatim_2i_0o(hex"b1", calldataload(4), calldataload(36))
        stop()
      }

      // storeFromTransient(uint256 slot) — берёт значение из transient storage слота 0
      case 0xd23ce946 {
        let v := tload(0)
        verbatim_2i_0o(hex"b1", calldataload(4), v)
        stop()
      }

      // load(uint256 slot) → uint256
      case 0x99d548aa {
        let v := verbatim_1i_1o(hex"b0", calldataload(4))
        mstore(0, v)
        return(0, 32)
      }

      // minSlot(uint256[] slots) → uint256
      case 0xfc3ca417 {
        let arrStart := add(4, calldataload(4))
        let n := calldataload(arrStart)
        let base := add(arrStart, 32)
        let best := 0
        let bestVal := not(0)
        let i := 0
        for { } lt(i, n) { i := add(i, 1) } {
          let slot := calldataload(add(base, mul(i, 32)))
          let v := verbatim_1i_1o(hex"b0", slot)
          if lt(v, bestVal) { bestVal := v best := slot }
        }
        mstore(0, best)
        return(0, 32)
      }

      // sumSlots(uint256[] slots) → uint256
      case 0x67e9187e {
        let arrStart := add(4, calldataload(4))
        let n := calldataload(arrStart)
        let base := add(arrStart, 32)
        let sum := 0
        let i := 0
        for { } lt(i, n) { i := add(i, 1) } {
          let slot := calldataload(add(base, mul(i, 32)))
          sum := add(sum, verbatim_1i_1o(hex"b0", slot))
        }
        mstore(0, sum)
        return(0, 32)
      }

      default { revert(0, 0) }
    }
  }
}
