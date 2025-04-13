use ::std::{borrow::Cow, fmt::Display, path::Path};

use ::bytesize::ByteSize;
use ::insensitive_buf::Insensitive;

use crate::{HashArray, Key, cli::Filter, fmt_oneshot::FmtOneshot};

#[derive(Clone, Copy)]
pub struct GroupSummary<'a> {
    name: Option<&'a Insensitive>,
    size: Option<u64>,
    hash: Option<&'a HashArray>,
    items: &'a [Cow<'a, Path>],
}

impl<'a> GroupSummary<'a> {
    pub fn new(key: &'a Key<'a>, items: &'a [Cow<'a, Path>], filter: &Filter) -> Self {
        Self {
            name: filter.name.is_yes().then_some(&key.name),
            size: filter.size.is_yes().then_some(key.size),
            hash: filter.hash.is_yes().then_some(&key.hash),
            items,
        }
    }
}

impl Display for GroupSummary<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut dstruct = f.debug_struct("");

        if let Some(name) = self.name {
            dstruct.field(
                "name",
                &FmtOneshot::new(|f| write!(f, "{}", name.display())),
            );
        }

        if let Some(size) = self.size {
            dstruct.field(
                "size",
                &FmtOneshot::new(|f| write!(f, "{}", ByteSize(size))),
            );
        }

        if let Some(hash) = self.hash {
            dstruct.field("hash", &FmtOneshot::new(|f| write!(f, "{:x}", hash)));
        }

        dstruct.field("item-count", &self.items.len());

        dstruct.field(
            "items",
            &FmtOneshot::new(|f| {
                let mut dset = f.debug_set();

                for path in self.items {
                    dset.entry(&FmtOneshot::new(|f| write!(f, "{}", path.display())));
                }

                dset.finish()
            }),
        );

        dstruct.finish()
    }
}
