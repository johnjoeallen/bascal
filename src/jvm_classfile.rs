//! Small internal JVM class-file writer used for the first direct-generation slice.
//!
//! The full JVM backend still falls back to Krakatau for general assembly. This
//! writer handles the minimal class shape (a default constructor and an empty
//! `main` method), allowing builds that do not need any runtime instructions to
//! produce a `.class` without an external assembler.

fn u2(out: &mut Vec<u8>, value: u16) {
    out.extend_from_slice(&value.to_be_bytes());
}
fn u4(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_be_bytes());
}

/// Returns bytes when `source` contains only a return-only JVM main method.
pub(crate) fn generate_return_only(source: &str) -> Option<Vec<u8>> {
    let class = source
        .lines()
        .find_map(|l| l.strip_prefix(".class public "))?
        .trim();
    let main = source
        .split(".method public static main : ([Ljava/lang/String;)V")
        .nth(1)?;
    let body = main.split(".end method").next()?;
    let instructions: Vec<_> = body
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with(".limit"))
        .collect();
    if instructions != ["return"] || source.matches(".method ").count() != 1 {
        return None;
    }

    // Constant pool: Object, its constructor, Code, main, descriptors, and class name.
    let mut out = Vec::new();
    u4(&mut out, 0xCAFEBABE);
    u2(&mut out, 0);
    u2(&mut out, 50);
    u2(&mut out, 12);
    out.push(1);
    u2(&mut out, 16);
    out.extend_from_slice(b"java/lang/Object"); // 1
    out.extend([7]);
    u2(&mut out, 1); // 2
    out.push(1);
    u2(&mut out, 6);
    out.extend_from_slice(b"<init>"); // 3
    out.push(1);
    u2(&mut out, 3);
    out.extend_from_slice(b"()V"); // 4
    out.extend([12]);
    u2(&mut out, 3);
    u2(&mut out, 4); // 5
    out.extend([10]);
    u2(&mut out, 2);
    u2(&mut out, 5); // 6
    out.push(1);
    u2(&mut out, 4);
    out.extend_from_slice(b"Code"); // 7
    out.push(1);
    u2(&mut out, class.len() as u16);
    out.extend_from_slice(class.as_bytes()); // 8
    out.extend([7]);
    u2(&mut out, 8); // 9
    out.push(1);
    u2(&mut out, 4);
    out.extend_from_slice(b"main"); // 10
    out.push(1);
    u2(&mut out, 22);
    out.extend_from_slice(b"([Ljava/lang/String;)V"); // 11
    u2(&mut out, 0x0021);
    u2(&mut out, 9);
    u2(&mut out, 2); // access, this, super
    u2(&mut out, 0);
    u2(&mut out, 0); // interfaces, fields
    u2(&mut out, 2); // methods
                     // <init>: aload_0, invokespecial Object.<init>, return
    u2(&mut out, 0x0001);
    u2(&mut out, 3);
    u2(&mut out, 4);
    u2(&mut out, 1);
    u2(&mut out, 7);
    u4(&mut out, 17);
    u2(&mut out, 1);
    u2(&mut out, 1);
    u4(&mut out, 5);
    out.extend([0x2a, 0xb7, 0, 6, 0xb1]);
    u2(&mut out, 0);
    u2(&mut out, 0);
    // main: return
    u2(&mut out, 0x0009);
    u2(&mut out, 10);
    u2(&mut out, 11);
    u2(&mut out, 1);
    u2(&mut out, 7);
    u4(&mut out, 13);
    u2(&mut out, 0);
    u2(&mut out, 1);
    u4(&mut out, 1);
    out.push(0xb1);
    u2(&mut out, 0);
    u2(&mut out, 0);
    u2(&mut out, 0); // class attributes
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::generate_return_only;

    #[test]
    fn writes_a_valid_return_only_class_header() {
        let assembly = ".version 50 0\n.class public Demo\n.super java/lang/Object\n\n.method public static main : ([Ljava/lang/String;)V\n    .limit stack 0\n    .limit locals 1\n\n    return\n.end method\n";
        let bytes = generate_return_only(assembly).expect("assembly should be recognized");
        assert_eq!(&bytes[..4], b"\xca\xfe\xba\xbe");
        assert_eq!(&bytes[4..8], &[0, 0, 0, 50]);
    }
}
