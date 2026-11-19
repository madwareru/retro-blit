use std::ops::Range;

pub(crate) fn find_dirty_row_spans(
    current: &[u8],
    uploaded: &[u8],
    width: usize,
    max_unchanged_gap: usize,
    output: &mut Vec<Range<usize>>,
) {
    output.clear();
    assert_eq!(current.len(), uploaded.len());
    assert!(width > 0);
    assert_eq!(current.len() % width, 0);

    let mut span_start = None;
    let mut last_dirty_row = 0;

    for (row, (current_row, uploaded_row)) in current
        .chunks_exact(width)
        .zip(uploaded.chunks_exact(width))
        .enumerate()
    {
        if current_row == uploaded_row {
            continue;
        }

        if let Some(start) = span_start {
            if row - last_dirty_row - 1 > max_unchanged_gap {
                output.push(start..last_dirty_row + 1);
                span_start = Some(row);
            }
        } else {
            span_start = Some(row);
        }
        last_dirty_row = row;
    }

    if let Some(start) = span_start {
        output.push(start..last_dirty_row + 1);
    }
}
