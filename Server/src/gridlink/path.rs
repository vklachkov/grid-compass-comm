use bstr::BStr;

use super::FrameError;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Path<'a> {
    pub server: &'a BStr,
    pub components: Vec<&'a BStr>,
}

impl<'a> Path<'a> {
    pub fn try_from_slice(data: &'a [u8]) -> Result<Self, FrameError> {
        let path = data
            .strip_prefix(b"`")
            .ok_or_else(|| FrameError::Validation {
                reason: "path must start with `".to_owned(),
            })?;

        let Some(separator) = path.iter().position(|&byte| byte == b':') else {
            return Err(FrameError::Validation {
                reason: "path must contain a server name followed by :".to_owned(),
            });
        };

        let server = &path[..separator];
        let resource = &path[separator + 1..];

        if server.is_empty() {
            return Err(FrameError::Validation {
                reason: "path server name must not be empty".to_owned(),
            });
        }

        let components = resource
            .split(|&byte| byte == b'`')
            .map(|component| {
                if component.is_empty() {
                    Err(FrameError::Validation {
                        reason: "path components must not be empty".to_owned(),
                    })
                } else {
                    Ok(BStr::new(component))
                }
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            server: BStr::new(server),
            components,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::Path;

    use bstr::BStr;

    #[test]
    fn parses_resource_path() {
        let raw = b"`vklachkov server:Name Device`Resources~Subject~";
        let path = Path::try_from_slice(raw).unwrap();

        assert_eq!(path.server, b"vklachkov server");
        assert_eq!(
            path.components,
            vec![BStr::new(b"Name Device"), BStr::new(b"Resources~Subject~")]
        );
    }

    #[test]
    fn rejects_invalid_path() {
        for path in [
            b"vklachkov server:Vfs~Manager~".as_slice(),
            b"`vklachkov server".as_slice(),
            b"`:Vfs~Manager~".as_slice(),
            b"`vklachkov server:".as_slice(),
            b"`vklachkov server:Name Device``Resources~Subject~".as_slice(),
        ] {
            assert!(Path::try_from_slice(path).is_err(), "{path:?}");
        }
    }
}
