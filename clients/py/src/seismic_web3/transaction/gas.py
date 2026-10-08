"""Internal gas-limit bounds for the current Seismic transaction pool."""


def seismic_pool_gas_minimum(
    encrypted_data: bytes,
    *,
    is_create: bool = False,
    authorization_count: int = 0,
) -> int:
    """Calculate the Prague admission minimum from final write ciphertext.

    Includes intrinsic gas, CREATE/initcode and EIP-7702 charges, and the
    EIP-7623 calldata floor. Seismic ignores access lists. The Prague bound is
    conservative on older nodes; this is not an execution estimate. Do not use
    the separately encrypted signed-read estimation twin's ciphertext here.
    """
    zero_bytes = encrypted_data.count(0)
    tokens = zero_bytes + 4 * (len(encrypted_data) - zero_bytes)
    intrinsic = 21_000 + 4 * tokens + 25_000 * authorization_count
    if is_create:
        intrinsic += 32_000 + 2 * ((len(encrypted_data) + 31) // 32)
    floor = 21_000 + 10 * tokens
    return max(intrinsic, floor)
