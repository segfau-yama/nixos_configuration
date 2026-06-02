use std::{
    io::{self, Write},
    process::{Command, Stdio},
};

pub fn hash_password(password: &str) -> io::Result<String> {
    match hash_with_mkpasswd(password) {
        Ok(hash) => Ok(hash),
        Err(mkpasswd_error) if mkpasswd_error.kind() == io::ErrorKind::NotFound => {
            hash_with_openssl(password)
        }
        Err(error) => Err(error),
    }
}

fn hash_with_mkpasswd(password: &str) -> io::Result<String> {
    let output = Command::new("mkpasswd")
        .args(["-m", "yescrypt", password])
        .output()?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ))
    }
}

fn hash_with_openssl(password: &str) -> io::Result<String> {
    let mut child = Command::new("openssl")
        .args(["passwd", "-6", "-stdin"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(password.as_bytes())?;
    }

    let output = child.wait_with_output()?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(io::Error::other(
            String::from_utf8_lossy(&output.stderr).trim().to_string(),
        ))
    }
}
