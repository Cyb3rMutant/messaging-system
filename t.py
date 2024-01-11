from os import urandom


def genkey(length: int) -> bytes:
    """Generate key."""
    return urandom(length)


def xor_strings(s, t) -> bytes:
    """Concate xor two strings together."""
    if isinstance(s, str):
        # Text strings contain single characters
        return "".join(chr(ord(a) ^ b) for a, b in zip(s, t)).encode("utf8")
    else:
        # Bytes objects contain integer values in the range 0-255
        return bytes([a ^ b for a, b in zip(s, t)])


message = "This is a secret message"
print("Message:", message)

key = genkey(len(message))
print("Key:", key)

cipherText = xor_strings(message.encode("utf8"), key)
print("cipherText:", cipherText)
print("decrypted:", xor_strings(cipherText, key).decode("utf8"))

# Verify
if xor_strings(cipherText, key).decode("utf8") == message:
    print("Unit test passed")
else:
    print("Unit test failed")


# Step 1: Parameters Setup
p = 23  # Prime number
g = 5  # Primitive root modulo p

# Step 2: Key Generation
a = 6  # Private key for party A
b = 15  # Private key for party B

A = pow(g, a, p)  # Public key for party A
B = pow(g, b, p)  # Public key for party B

# Exchange public keys (A and B) over the communication channel

# Step 3: Shared Secret Calculation
shared_secret_A = pow(B, a, p)  # Calculated by party A
shared_secret_B = pow(A, b, p)  # Calculated by party B

# Step 4: Both parties now have the same shared secret
print("Shared Secret for A:", shared_secret_A)
print("Shared Secret for B:", shared_secret_B)
