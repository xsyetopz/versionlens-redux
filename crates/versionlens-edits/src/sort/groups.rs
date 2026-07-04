use super::slots::{SortSlot, dependency_group};

pub(super) fn grouped_slots<'a>(
    lines: &[&str],
    slots: Vec<SortSlot<'a>>,
) -> Vec<(&'a str, Vec<SortSlot<'a>>)> {
    let mut groups = Vec::<(&str, Vec<SortSlot<'a>>)>::new();

    for slot in slots {
        let group = dependency_group(slot.dependency);
        if let Some((last_group, group_slots)) = groups.last_mut()
            && *last_group == group
            && !has_block_boundary(lines, group_slots.last(), &slot)
        {
            group_slots.push(slot);
            continue;
        }

        groups.push((group, vec![slot]));
    }

    groups
}

fn has_block_boundary(
    lines: &[&str],
    previous: Option<&SortSlot<'_>>,
    next: &SortSlot<'_>,
) -> bool {
    let Some(previous) = previous else {
        return false;
    };

    if previous.end + 1 >= next.start {
        return false;
    }

    lines[previous.end + 1..next.start]
        .iter()
        .map(|line| line.trim())
        .any(is_block_boundary)
}

fn is_block_boundary(line: &str) -> bool {
    line == ")" || line == "end" || line.ends_with('(') || line.ends_with(" do")
}
