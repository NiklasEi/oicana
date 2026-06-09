use std::borrow::Cow;
use std::num::NonZeroUsize;

use serde::{Deserialize, Serialize};
use typst::introspection::Introspector;
use typst::layout::{PageRanges, PagedDocument};

/// A contiguous, 0-based inclusive range of document pages.
///
/// Both bounds are optional; `None` means the bound is open. For example, a
/// range with `start: Some(1), end: None` selects the second page to the end of
/// the document, and the default (both `None`) selects every page.
#[derive(Serialize, Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct PageRange {
    /// First selected page index (0-based, inclusive). `None` selects from the
    /// first page.
    pub start: Option<usize>,
    /// Last selected page index (0-based, inclusive). `None` selects up to the
    /// last page.
    pub end: Option<usize>,
}

impl PageRange {
    /// A range selecting exactly the page at the given 0-based index.
    pub fn single(page: usize) -> Self {
        Self {
            start: Some(page),
            end: Some(page),
        }
    }

    /// A range selecting from the given 0-based page index to the end of the
    /// document.
    pub fn from(start: usize) -> Self {
        Self {
            start: Some(start),
            end: None,
        }
    }

    /// A range selecting from the first page up to (and including) the given
    /// 0-based page index.
    pub fn to(end: usize) -> Self {
        Self {
            start: None,
            end: Some(end),
        }
    }

    /// The 0-based indices selected from a document with `page_count` pages:
    /// ascending and clamped to the in-bounds pages. Empty when the range lies
    /// entirely outside the document.
    pub fn selected_indices(&self, page_count: usize) -> Vec<usize> {
        if page_count == 0 {
            return Vec::new();
        }
        let start = self.start.unwrap_or(0);
        let end = self.end.unwrap_or(page_count - 1).min(page_count - 1);
        if start > end {
            return Vec::new();
        }
        (start..=end).collect()
    }
}

/// Convert a [`PageRange`] into a Typst [`PageRanges`] holding this single
/// range. Typst page ranges are 1-based inclusive, so our 0-based bounds are
/// shifted up by one.
impl From<&PageRange> for PageRanges {
    fn from(range: &PageRange) -> Self {
        let to_one_based = |index: usize| {
            NonZeroUsize::new(index + 1).expect("0-based page index plus one is never zero")
        };
        let start = range.start.map(to_one_based);
        let end = range.end.map(to_one_based);
        PageRanges::new(vec![start..=end])
    }
}

/// Borrow the whole `document` when `page` is `None`, otherwise build a new
/// document containing only the selected pages (cloned) plus a default
/// introspector.
pub fn select_pages<'a>(
    document: &'a PagedDocument,
    page: Option<&PageRange>,
) -> Cow<'a, PagedDocument> {
    match page {
        None => Cow::Borrowed(document),
        Some(range) => {
            let pages = range
                .selected_indices(document.pages.len())
                .into_iter()
                .map(|index| document.pages[index].clone())
                .collect();
            Cow::Owned(PagedDocument {
                pages,
                info: document.info.clone(),
                introspector: Introspector::default(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_range_selects_all_pages() {
        let range = PageRange::default();
        assert_eq!(range.selected_indices(3), vec![0, 1, 2]);
    }

    #[test]
    fn single_page_selects_one_index() {
        assert_eq!(PageRange::single(2).selected_indices(3), vec![2]);
    }

    #[test]
    fn open_start_and_open_end() {
        assert_eq!(PageRange::to(1).selected_indices(3), vec![0, 1]);
        assert_eq!(PageRange::from(1).selected_indices(3), vec![1, 2]);
    }

    #[test]
    fn range_is_clamped_to_document() {
        let range = PageRange {
            start: Some(1),
            end: Some(10),
        };
        assert_eq!(range.selected_indices(3), vec![1, 2]);
    }

    #[test]
    fn out_of_bounds_range_is_empty() {
        assert_eq!(PageRange::from(3).selected_indices(3), Vec::<usize>::new());
        assert_eq!(
            PageRange::single(3).selected_indices(3),
            Vec::<usize>::new()
        );
    }

    #[test]
    fn empty_document_selects_nothing() {
        assert!(PageRange::default().selected_indices(0).is_empty());
    }
}
