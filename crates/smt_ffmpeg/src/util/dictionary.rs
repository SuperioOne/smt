use crate::{common::unsafe_av_result, error::AvError};
use smt_ffmpeg_sys::{
  // AV_DICT_DEDUP,
  AV_DICT_IGNORE_SUFFIX,
  AV_DICT_MULTIKEY,
  AVDictionary,
  AVDictionaryEntry,
  av_dict_count,
  av_dict_free,
  av_dict_get,
  av_dict_iterate,
  av_dict_set,
};

const AV_DICT_DEDUP: u32 = 128;

use std::{
  borrow::{Borrow, Cow},
  ffi::{CStr, CString},
  iter::Flatten,
  ptr::{null, null_mut},
  str::FromStr,
};

pub struct AvDictionaryRef<'a> {
  inner: &'a *mut AVDictionary,
}

// NOTE: Indirection is required to initialize empty dictionary (null pointer)
pub struct AvDictionaryMut<'a> {
  inner: &'a mut *mut AVDictionary,
}

macro_rules! impl_shared_fns {
  ($lf:lifetime, $type:ty) => {
    impl<$lf> $type {
      pub fn len(&self) -> usize {
        if self.inner.is_null() {
          0
        } else {
          unsafe { av_dict_count(*self.inner) as usize }
        }
      }

      pub fn get<K>(&self, key: K) -> Option<&$lf CStr>
      where
        K: Borrow<CStr>,
      {
        if self.inner.is_null() {
          return None;
        }

        let mut tag: *const AVDictionaryEntry = null();
        tag = unsafe {
          av_dict_get(
            *self.inner,
            key.borrow().as_ptr(),
            tag,
            AV_DICT_IGNORE_SUFFIX as i32,
          )
        };

        unsafe {
          if tag.is_null() || (*tag).value.is_null() {
            None
          } else {
            Some(CStr::from_ptr((*tag).value))
          }
        }
      }

      pub fn has<K>(&self, key: K) -> bool
      where
        K: Borrow<CStr>,
      {
        if self.inner.is_null() {
          return false;
        }

        let mut tag: *const AVDictionaryEntry = null();

        tag = unsafe {
          av_dict_get(
            *self.inner,
            key.borrow().as_ptr(),
            tag,
            AV_DICT_IGNORE_SUFFIX as i32,
          )
        };

        unsafe {
          if tag.is_null() || (*tag).value.is_null() {
            false
          } else {
            true
          }
        }
      }

      pub fn iter(&$lf self) -> Iter<'a> {
        EntryIter {
            dictionary: self.inner,
            current: null()
        }.flatten()
      }
    }
  };
}

impl_shared_fns!('a, AvDictionaryRef<'a>);
impl_shared_fns!('a, AvDictionaryMut<'a>);

/// Random null-terminator in middle of a [`str`] is not expected.
const CSTR_PANIC_MESSAGE: &'static str = "unexpected null terminator on rust str";

fn escape_value(value: &str) -> Cow<'_, str> {
  let mut chars = value.chars().enumerate();
  let mut escaped = String::new();

  loop {
    match chars.next() {
      Some((idx, ';')) => {
        escaped.reserve(value.len() + 1);
        escaped.push_str(&value[..idx]);
        escaped.push_str("\\;");
        break;
      }
      None => return Cow::Borrowed(value),
      _ => continue,
    }
  }

  for (_, ch) in chars {
    match ch {
      ';' => {
        escaped.push_str("\\;");
      }
      v => {
        escaped.push(v);
      }
    }
  }

  Cow::Owned(escaped)
}

fn unescape_value(value: &str) -> Cow<'_, str> {
  if value.contains("\\;") {
    Cow::Owned(value.replace("\\;", ";"))
  } else {
    Cow::Borrowed(value)
  }
}

impl<'a> AvDictionaryRef<'a> {
  #[inline]
  pub const fn from_ptr_ref(dictionary: &'a *mut AVDictionary) -> Self {
    Self { inner: dictionary }
  }
}

impl<'a> AvDictionaryMut<'a> {
  #[inline]
  pub const fn from_ptr_ref(dictionary: &'a mut *mut AVDictionary) -> Self {
    Self { inner: dictionary }
  }

  pub fn set<K, V>(&mut self, key: K, value: V) -> Result<(), AvError>
  where
    K: Borrow<str>,
    V: Borrow<str>,
  {
    let key = CString::from_str(key.borrow()).expect(CSTR_PANIC_MESSAGE);
    let value = CString::from_str(&escape_value(value.borrow())).expect(CSTR_PANIC_MESSAGE);
    unsafe { self.set_cstr(key, value) }
  }

  pub fn push<K, V>(&mut self, key: K, value: V) -> Result<(), AvError>
  where
    K: Borrow<str>,
    V: Borrow<str>,
  {
    let key = CString::from_str(key.borrow()).expect(CSTR_PANIC_MESSAGE);
    let value = CString::from_str(&escape_value(value.borrow())).expect(CSTR_PANIC_MESSAGE);
    unsafe { self.push_cstr(key, value) }
  }

  pub fn clear(&mut self) {
    unsafe {
      av_dict_free(self.inner);
    }
    *self.inner = null_mut();
  }

  pub unsafe fn set_cstr<K, V>(&mut self, key: K, value: V) -> Result<(), AvError>
  where
    K: Borrow<CStr>,
    V: Borrow<CStr>,
  {
    unsafe_av_result!(av_dict_set(
      self.inner,
      key.borrow().as_ptr(),
      value.borrow().as_ptr(),
      0
    ))
  }

  pub unsafe fn push_cstr<K, V>(&mut self, key: K, value: V) -> Result<(), AvError>
  where
    K: Borrow<CStr>,
    V: Borrow<CStr>,
  {
    unsafe_av_result!(av_dict_set(
      self.inner,
      key.borrow().as_ptr(),
      value.borrow().as_ptr(),
      (AV_DICT_DEDUP | AV_DICT_MULTIKEY) as i32
    ))
  }
}

pub type Iter<'a> = Flatten<EntryIter<'a>>;

pub struct EntryIter<'a> {
  dictionary: &'a *mut AVDictionary,
  current: *const AVDictionaryEntry,
}

pub struct ValueIter<'a> {
  key: &'a str,
  value: &'a str,
  cursor: usize,
}

impl<'a> Iterator for EntryIter<'a> {
  type Item = ValueIter<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    loop {
      self.current = unsafe { av_dict_iterate(*self.dictionary, self.current) };
      let tag = self.current;

      unsafe {
        if tag.is_null() {
          return None;
        } else {
          if (*tag).key.is_null() || (*tag).value.is_null() {
            // Kinda impossible case, skips current entry if tag or it's key is null
            continue;
          } else {
            let key = CStr::from_ptr((*tag).key).to_str();
            let value = CStr::from_ptr((*tag).value).to_str();

            match (key, value) {
              (Ok(key), Ok(value)) => {
                return Some(ValueIter {
                  key,
                  value,
                  cursor: 0,
                });
              }
              _ => continue,
            }
          }
        }
      }
    }
  }
}

impl<'a> Iterator for ValueIter<'a> {
  type Item = (&'a str, Cow<'a, str>);

  fn next(&mut self) -> Option<Self::Item> {
    let start = self.cursor;

    if start >= self.value.len() {
      return None;
    }

    let target = &self.value[start..];
    let mut end: Option<usize> = None;
    let mut chars = target.chars();

    'SEARCH: loop {
      match chars.next() {
        Some(';') => {
          end = Some(self.cursor);
          self.cursor += 1;
          break 'SEARCH;
        }
        Some('\\') => {
          self.cursor += 1;
          if let Some(ch) = chars.next() {
            self.cursor += ch.len_utf8();
          }
        }
        Some(ch) => {
          self.cursor += ch.len_utf8();
        }
        None => {
          self.cursor += 1;
          break 'SEARCH;
        }
      }
    }

    let value_slice = match end {
      Some(end) => &self.value[start..end].trim(),
      None => &self.value[start..].trim(),
    };

    Some((&self.key, unescape_value(*value_slice)))
  }
}
