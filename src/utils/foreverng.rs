// !!! WARNING: READ THE WARNING !!!
// You'll notice this module sort of implements AES-CBC.
// DO NOT USE IT FOR ANYTHING THAT NEEDS SECURITY. HOLY SHIT. NO. NEVER. NOT EVEN ONCE.
// AES and SHA are only in here because they reliably produce random-looking output.
// NONE of the code in this project depends on any cryptographic guarantees from this type.
// None of yours ever should, either.
// Ok?
// Ok.

use {
  crypto::{
    aes::KeySize, aesni, md5,
    digest::Digest as _, symmetriccipher::BlockEncryptor as _
  },
  rand::{Error, RngCore},
};

const KEY_SZ: usize = 128 / 8; // MD5 outputs 128-bit keys so
const BLOCK_SZ: usize = 16;

fn chunks(seed: &[u8]) -> Vec<[u8; BLOCK_SZ]> {
  let mut res = Vec::with_capacity(seed.len() / BLOCK_SZ + 1);
  for start in (0..seed.len()).step_by(BLOCK_SZ) {
    let end = BLOCK_SZ.min(seed.len() - start);
    let in_chunk = &seed[start..end];
    let mut out_chunk = [0; BLOCK_SZ];
    out_chunk[..in_chunk.len()].copy_from_slice(in_chunk);
    res.push(out_chunk);
  }
  res
}

fn xor(data: &mut [u8], key: &[u8]) {
  for (i, byte) in key.iter().enumerate() {
    data[i] ^= byte;
  }
}

fn seed_round(enc: &aesni::AesNiEncryptor, last: &[u8], mut seed: [u8; BLOCK_SZ]) -> [u8; BLOCK_SZ] {
  let mut output = [0; BLOCK_SZ];
  xor(&mut seed, last);
  enc.encrypt_block(&seed, &mut output);
  output
}

/// A non-secure PRNG which can be seeded with as much data as you want, and will produce different outputs for
/// different seeds. Note that you can only seed once, at the beginning, but with as much data as you want. Strictly
/// speaking, there are collisions: The internal state is 128 bits. That's a LOT of space before collisions are
/// likely, though, for artsy random data.
/// 
/// Note that despite the internal use of cryptographic primitives, this is an **insecure** random number generator.
/// AES was only chosen because it produces very random-looking output and, with AESNI, it runs quickly too.
/// 
/// Internally, this more or less does AES-CBC, with an all-zero IV, using the MD5 of the seed as the key. First it
/// encrypts the contents of the key itself, to get an initial state. Once it has that, it produces random bytes by
/// encrypting infinite zeroes.
/// 
/// Reseeding is similar; it uses the internal state of the parent for the IV instead of an all-zero one.
pub struct ForeveRNG {
  next: [u8; BLOCK_SZ],
  left: usize,
  enc: aesni::AesNiEncryptor,
}

impl ForeveRNG {
  fn new(iv: [u8; BLOCK_SZ], seed: &[u8]) -> ForeveRNG {
    // Key come from the SHA-3-256 of the whole key, so that keys identical except different amounts of trailing
    // zeroes (otherwise swallowed by padding) will still produce very different output
    let mut hasher = md5::Md5::new();
    hasher.input(seed);
    let mut key = [0; KEY_SZ];
    hasher.result(&mut key);
    let enc = aesni::AesNiEncryptor::new(KeySize::KeySize128, &key);
    // Then we encrypt the contents of the seed, so that hash collisions don't produce identical output
    let mut data = iv;
    for chunk in chunks(seed) {
      data = seed_round(&enc, &data, chunk);
    }
    ForeveRNG {
      next: data,
      left: data.len(),
      enc
    }
  }

  /// Create a new ForeveRNG with the given seed
  pub fn with_seed(seed: &[u8]) -> ForeveRNG {
    Self::new([0; BLOCK_SZ], seed)
  }

  /// Base a new ForeveRNG on an existing one, with new seed data
  pub fn reseed(&self, new_seed: &[u8]) -> ForeveRNG {
    // Note: This _deliberately_ does not advance self.next, even if it's partially consumed.
    // That way multiple reseeds with the same new seed produce the same sequence of values.
    // The reused randomness isn't important because it's "shuffled in" to the rest, so it won't lead to repeats.
    Self::new(self.next, new_seed)
  }

  fn refill(&mut self) {
    let mut new_data = [0; BLOCK_SZ];
    self.enc.encrypt_block(&self.next, &mut new_data);
    self.next = new_data;
    self.left = BLOCK_SZ;
  }
}

impl RngCore for ForeveRNG {
  fn fill_bytes(&mut self, dest: &mut [u8]) {
    if dest.len() == 0 {
      // very easy: nothing to copy
      return;
    }
    if dest.len() < self.left {
      // pretty easy: copy from our stash of generated bytes
      let start = self.next.len() - self.left;
      let end = start + dest.len();
      dest.copy_from_slice(&self.next[start..end]);
      self.left -= dest.len();
      return;
    }
    // less easy
    let mut pos = 0;
    // first, exhaust our current cache, if we have any
    if self.left > 0 {
      let start = self.next.len() - self.left;
      dest[0..self.left].copy_from_slice(&self.next[start..]);
      pos += self.left;
      self.left -= 0;
    }
    // then, as long as there's more than one chunk size left...
    while dest.len() - pos > BLOCK_SZ {
      // fill up our cache
      self.refill();
      // then copy the whole thing into the destination
      dest[pos..pos+BLOCK_SZ].copy_from_slice(&self.next);
      pos += BLOCK_SZ;
    }
    // then just copy over the last little bit remaining
    self.refill();
    let final_copy = dest.len() - pos;
    dest[pos..].copy_from_slice(&self.next[..final_copy]);
  }

  fn next_u32(&mut self) -> u32 {
    let mut bytes = [0; 4];
    self.fill_bytes(&mut bytes);
    u32::from_ne_bytes(bytes)
  }

  fn next_u64(&mut self) -> u64 {
    let mut bytes = [0; 8];
    self.fill_bytes(&mut bytes);
    u64::from_ne_bytes(bytes)
  }

  fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), Error> {
    self.fill_bytes(dest);
    Ok(())
  }
}
