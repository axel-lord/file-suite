use ::std::{ffi::OsStr, os::unix::ffi::OsStrExt as _};

use ::chumsky::extra;

use crate::Error;

#[derive(Debug)]
pub enum LookupChunk<'a, I: ?Sized> {
    Text(&'a OsStr),
    CaptureIdx(usize),
    CaptureName(&'a I),
    CaptureIdxOpt(usize),
    CaptureNameOpt(&'a I),
}

impl<'a, I: ?Sized> Copy for LookupChunk<'a, I> {}

impl<'a, I: ?Sized> Clone for LookupChunk<'a, I> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a> LookupChunk<'a, str> {
    pub fn from_chunks<C, I>(chunks: I) -> Result<C, Error>
    where
        C: FromIterator<Self>,
        I: IntoIterator<Item = ::parse_fmt::Chunk<'a>>,
    {
        let parser = LookupChunk::lookup_parser();
        let mut idx_counter = 0;

        chunks
            .into_iter()
            .map(|chunk| {
                LookupChunk::from_chunk_u8(chunk, &mut idx_counter, &parser)
                    .and_then(|chunk| chunk.to_utf8())
            })
            .collect()
    }
}

type Extra<'a> = extra::Err<::chumsky::error::Rich<'a, u8>>;

impl<'a> LookupChunk<'a, [u8]> {
    pub fn lookup_parser() -> impl ::chumsky::Parser<'a, &'a [u8], Self, Extra<'a>> {
        use ::chumsky::prelude::*;

        let num = text::int(10)
            .map(|s: &[u8]| LookupChunk::CaptureIdx(str::from_utf8(s).unwrap().parse().unwrap()));
        let opt_num = text::int(10).map(|s: &[u8]| {
            LookupChunk::CaptureIdxOpt(str::from_utf8(s).unwrap().parse().unwrap())
        });
        let name = any()
            .repeated()
            .to_slice()
            .map(|s: &[u8]| LookupChunk::CaptureName(s));
        let opt_name = any()
            .repeated()
            .to_slice()
            .map(|s: &[u8]| LookupChunk::CaptureNameOpt(s));

        let opt = opt_num.or(opt_name);
        let non_opt = num.or(name);

        choice((
            just(b'?').ignore_then(opt),
            just(b'.').ignore_then(non_opt),
            just(b'#').ignore_then(choice((
                just(b'?').ignore_then(opt_num),
                just(b'.').ignore_then(num),
                num,
            ))),
            just(b'-').ignore_then(choice((
                just(b'?').ignore_then(opt_name),
                just(b'.').ignore_then(name),
                name,
            ))),
            non_opt,
        ))
    }

    fn from_chunk_u8(
        chunk: ::parse_fmt::Chunk<'a>,
        idx_counter: &mut usize,
        lookup_parser: &impl ::chumsky::Parser<'a, &'a [u8], Self, Extra<'a>>,
    ) -> Result<Self, Error> {
        match chunk {
            ::parse_fmt::Chunk::Text(items) => Ok(Self::Text(OsStr::from_bytes(items))),
            ::parse_fmt::Chunk::Lookup(items) => {
                match lookup_parser.parse(items).into_result().map_err(|err| {
                    use ::std::fmt::Write as _;
                    let mut msg = String::new();

                    for err in err {
                        write!(msg, "\n{err}").expect("write to string should succeed");
                    }

                    Error::ParseLookup {
                        chunk: items.into(),
                        msg,
                    }
                })? {
                    Self::CaptureName(name) if name.is_empty() => {
                        let val = Self::CaptureIdx(*idx_counter);
                        *idx_counter += 1;
                        Ok(val)
                    }
                    Self::CaptureNameOpt(name) if name.is_empty() => {
                        let val = Self::CaptureIdxOpt(*idx_counter);
                        *idx_counter += 1;
                        Ok(val)
                    }
                    other => Ok(other),
                }
            }
        }
    }

    pub fn to_utf8(self) -> Result<LookupChunk<'a, str>, Error> {
        Ok(match self {
            LookupChunk::Text(os_str) => LookupChunk::Text(os_str),
            LookupChunk::CaptureIdx(idx) => LookupChunk::CaptureIdx(idx),
            LookupChunk::CaptureName(name) => LookupChunk::CaptureName(
                str::from_utf8(name)
                    .map_err(|err| Error::NonUtf8CaptureLookup(err, name.into()))?,
            ),
            LookupChunk::CaptureIdxOpt(idx) => LookupChunk::CaptureIdxOpt(idx),
            LookupChunk::CaptureNameOpt(name) => LookupChunk::CaptureNameOpt(
                str::from_utf8(name)
                    .map_err(|err| Error::NonUtf8CaptureLookup(err, name.into()))?,
            ),
        })
    }
}
