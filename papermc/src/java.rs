use jni::objects::{JObject, JObjectArray, JString, JValue};
use jni::{jni_sig, jni_str};

use crate::api::Api;
use crate::jobject_repr::JObjectRepr;
use crate::{papermc_jobject, papermc_jobject_inst};

/// Convert a Java value into a Rust value.
///
/// A JNI-aware analogue of [std::convert::TryFrom].
///
/// See also [ToJava] for reverse conversion.
pub trait FromJava<'local, J>: Sized {
    fn from_java(api: &mut Api<'_, 'local>, source: &J) -> eyre::Result<Self>;
}

/// Convert a Rust value into a Java value.
///
/// The reverse of [FromJava]; A JNI-aware analogue of [std::convert::TryInto].
pub trait ToJava<'local, J> {
    fn to_java(&self, api: &mut Api<'_, 'local>) -> eyre::Result<J>;
}

impl<'local> FromJava<'local, JString<'local>> for String {
    fn from_java(api: &mut Api<'_, 'local>, source: &JString<'local>) -> eyre::Result<Self> {
        Ok(source.try_to_string(api.jni())?)
    }
}

impl<'local> ToJava<'local, JString<'local>> for str {
    fn to_java(&self, api: &mut Api<'_, 'local>) -> eyre::Result<JString<'local>> {
        Ok(api.jni().new_string(self)?)
    }
}

/// Reads a Java `String[]` into owned Rust strings.
impl<'local> FromJava<'local, JObjectArray<'local, JString<'local>>> for Vec<String> {
    fn from_java(
        api: &mut Api<'_, 'local>,
        source: &JObjectArray<'local, JString<'local>>,
    ) -> eyre::Result<Self> {
        let env = api.jni();
        let len = source.len(env)?;
        // Each `get_element` allocates a local JNI ref. JNI guarantees only 16 locals by default,
        // so a long array overflows the caller's frame allotment. Push a sized sub-frame so those
        // intermediates are released en masse on return.
        Ok(
            env.with_local_frame(len + 4, |env| -> jni::errors::Result<Vec<String>> {
                let mut out = Vec::with_capacity(len);
                for i in 0..len {
                    let elem = source.get_element(env, i)?;
                    out.push(elem.try_to_string(env)?);
                }
                Ok(out)
            })?,
        )
    }
}

papermc_jobject_inst! {
    /// Mirrors `java.util.List`.
    ///
    /// See <https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/List.html>.
    pub ListInst<'local> = "java/util/List": List;
}

/// Mirrors `java.util.List`.
pub trait List<'local>: JObjectRepr<'local> {
    /// Mirrors `java.util.List#size()`.
    fn size(&self, api: &mut Api<'_, 'local>) -> eyre::Result<i32> {
        let env = api.jni();
        Ok(env
            .call_method(self.as_jobject(), jni_str!("size"), jni_sig!("()I"), &[])?
            .i()?)
    }

    /// Mirrors `java.util.List#get(int)`.
    fn get(&self, api: &mut Api<'_, 'local>, index: i32) -> eyre::Result<JObject<'local>> {
        let env = api.jni();
        Ok(env
            .call_method(
                self.as_jobject(),
                jni_str!("get"),
                jni_sig!("(I)Ljava/lang/Object;"),
                &[JValue::Int(index)],
            )?
            .l()?)
    }

    /// Mirrors `java.util.List#add(E)`.
    fn add(&self, api: &mut Api<'_, 'local>, element: &JObject<'_>) -> eyre::Result<bool> {
        let env = api.jni();
        Ok(env
            .call_method(
                self.as_jobject(),
                jni_str!("add"),
                jni_sig!("(Ljava/lang/Object;)Z"),
                &[JValue::Object(element)],
            )?
            .z()?)
    }
}

/// Reads a `java.util.List` of Java objects into wrapper values.
///
/// A list whose elements are not `T` produces wrappers whose method calls fail at runtime.
impl<'local, T: JObjectRepr<'local>> FromJava<'local, ListInst<'local>> for Vec<T> {
    fn from_java(api: &mut Api<'_, 'local>, source: &ListInst<'local>) -> eyre::Result<Self> {
        let size = source.size(api)?;
        let mut out = Vec::with_capacity(size as usize);
        for i in 0..size {
            let obj = source.get(api, i)?;
            out.push(unsafe { T::from_jobject(obj) });
        }
        Ok(out)
    }
}

papermc_jobject! {
    /// Mirrors `java.util.ArrayList`.
    ///
    /// See <https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/ArrayList.html>.
    pub ArrayList<'local> = "java/util/ArrayList": List;
}

impl<'local> ArrayList<'local> {
    /// Mirrors the `java.util.ArrayList#ArrayList(int)` constructor.
    pub fn new(api: &mut Api<'_, 'local>, initial_capacity: i32) -> eyre::Result<Self> {
        let env = api.jni();
        let obj = env.new_object(
            jni_str!("java/util/ArrayList"),
            jni_sig!("(I)V"),
            &[JValue::Int(initial_capacity)],
        )?;
        Ok(unsafe { Self::from_jobject(obj) })
    }
}

/// Builds a `java.util.ArrayList` sized to the slice.
impl<'local> ToJava<'local, ArrayList<'local>> for [String] {
    fn to_java(&self, api: &mut Api<'_, 'local>) -> eyre::Result<ArrayList<'local>> {
        let list = ArrayList::new(api, i32::try_from(self.len()).unwrap_or(i32::MAX))?;
        for s in self {
            let jstr = s.to_java(api)?;
            list.add(api, &jstr)?;
        }
        Ok(list)
    }
}

papermc_jobject! {
    /// Mirrors `java.util.Map`.
    ///
    /// See <https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/Map.html>.
    pub Map<'local> = "java/util/Map";
}

impl<'local> Map<'local> {
    /// Mirrors `java.util.Map#get(Object)`.
    ///
    /// Returns `None` where Java returns null: no mapping for `key`.
    pub fn get(
        &self,
        api: &mut Api<'_, 'local>,
        key: &JObject<'_>,
    ) -> eyre::Result<Option<JObject<'local>>> {
        let env = api.jni();
        let value = env
            .call_method(
                &self.obj,
                jni_str!("get"),
                jni_sig!("(Ljava/lang/Object;)Ljava/lang/Object;"),
                &[JValue::Object(key)],
            )?
            .l()?;
        Ok((!value.is_null()).then_some(value))
    }

    /// Mirrors `java.util.Map#remove(Object)`.
    ///
    /// Returns the removed value, or `None` where Java returns null: no mapping for `key`.
    pub fn remove(
        &self,
        api: &mut Api<'_, 'local>,
        key: &JObject<'_>,
    ) -> eyre::Result<Option<JObject<'local>>> {
        let env = api.jni();
        let value = env
            .call_method(
                &self.obj,
                jni_str!("remove"),
                jni_sig!("(Ljava/lang/Object;)Ljava/lang/Object;"),
                &[JValue::Object(key)],
            )?
            .l()?;
        Ok((!value.is_null()).then_some(value))
    }
}

papermc_jobject! {
    /// Mirrors `java.util.concurrent.Future`.
    ///
    /// See <https://docs.oracle.com/en/java/javase/21/docs/api/java.base/java/util/concurrent/Future.html>.
    pub Future<'local> = "java/util/concurrent/Future";
}

impl<'local> Future<'local> {
    /// Mirrors `java.util.concurrent.Future#get()`.
    ///
    /// Blocks the calling thread until the computation completes. Never call from the main
    /// server thread when the computation itself needs the main thread; that deadlocks.
    pub fn get(&self, api: &mut Api<'_, 'local>) -> eyre::Result<JObject<'local>> {
        let env = api.jni();
        Ok(env
            .call_method(
                &self.obj,
                jni_str!("get"),
                jni_sig!("()Ljava/lang/Object;"),
                &[],
            )?
            .l()?)
    }
}
