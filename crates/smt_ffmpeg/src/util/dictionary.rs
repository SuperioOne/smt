use crate::{common::unsafe_av_result, error::AvError};
use smt_ffmpeg_sys::{
  AV_DICT_IGNORE_SUFFIX, AV_DICT_MULTIKEY, AVDictionary, AVDictionaryEntry, av_dict_count,
  av_dict_get, av_dict_iterate, av_dict_set,
};
use std::{
  borrow::Borrow,
  ffi::{CStr, CString},
  ptr::null,
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

      pub fn iter(&$lf self) -> Iter<$lf> {
        Iter {
          dictionary: self.inner,
          current: null(),
        }
      }
    }
  };
}

impl_shared_fns!('a, AvDictionaryRef<'a>);

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

  pub fn override_entry<K, V>(&mut self, key: K, value: V) -> Result<(), AvError>
  where
    K: Borrow<str>,
    V: Borrow<str>,
  {
    self.internal_set(key, value, 0)
  }

  pub fn set<K, V>(&mut self, key: K, value: V) -> Result<(), AvError>
  where
    K: Borrow<str>,
    V: Borrow<str>,
  {
    self.internal_set(key, value, (AV_DICT_MULTIKEY) as i32)
  }

  fn internal_set<K, V>(&mut self, key: K, value: V, flag: i32) -> Result<(), AvError>
  where
    K: Borrow<str>,
    V: Borrow<str>,
  {
    let key = CString::from_str(key.borrow()).expect("unexpected null terminator on rust str");
    let value = CString::from_str(value.borrow()).expect("unexpected null terminator on rust str");

    unsafe_av_result!(av_dict_set(self.inner, key.as_ptr(), value.as_ptr(), flag))
  }
}

pub struct Iter<'a> {
  dictionary: &'a *mut AVDictionary,
  current: *const AVDictionaryEntry,
}

impl<'a> Iterator for Iter<'a> {
  type Item = (&'a CStr, &'a CStr);

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
            let key = CStr::from_ptr((*tag).key);
            let value = CStr::from_ptr((*tag).value);

            return Some((key, value));
          }
        }
      }
    }
  }
}
