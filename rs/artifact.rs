//! Complete, bounded and allocation-order-independent NOXDAG01 artifacts.
//! Network wire messages remain in `encode`; see specs/artifact.md.
use crate::data::Data;
use crate::encode::{self, DecodedData, Particle};
use crate::{Order, Reduction};
use alloc::{
    collections::{BTreeMap, BTreeSet},
    vec,
    vec::Vec,
};

const MAGIC: &[u8; 8] = b"NOXDAG01";
const HEADER: usize = 44;
const MIN_ENTRY: usize = 41;
const MODULUS: u64 = 0xffff_ffff_0000_0001;

#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub max_bytes: usize,
    pub max_nodes: u32,
    pub max_depth: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Error {
    Framing,
    Version,
    FieldRange,
    HashMismatch,
    Duplicate,
    MissingReference,
    MissingRoot,
    Order,
    InvalidNode,
    ByteLimit,
    NodeLimit,
    DepthLimit,
    Allocation,
}

fn particle(bytes: &[u8]) -> Result<Particle, Error> {
    let id: Particle = bytes.try_into().map_err(|_| Error::Framing)?;
    for limb in id.chunks_exact(8) {
        let v = u64::from_le_bytes(limb.try_into().map_err(|_| Error::Framing)?);
        if v >= MODULUS {
            return Err(Error::FieldRange);
        }
    }
    Ok(id)
}

fn bounded_depth(left: u32, right: u32, limits: Limits) -> Result<u32, Error> {
    let depth = left.max(right).checked_add(1).ok_or(Error::DepthLimit)?;
    if depth > limits.max_depth {
        return Err(Error::DepthLimit);
    }
    Ok(depth)
}

/// Encode a reachable DAG in first-visit left-first postorder, preserving sharing.
pub fn encode<const N: usize>(
    ar: &Reduction<N>,
    root: Order,
    limits: Limits,
) -> Result<Vec<u8>, Error> {
    if HEADER > limits.max_bytes {
        return Err(Error::ByteLimit);
    }
    let root_id = encode::particle_id(ar, root).ok_or(Error::InvalidNode)?;
    particle(&root_id)?;
    let mut out = Vec::new();
    out.extend_from_slice(MAGIC);
    out.extend_from_slice(&root_id);
    out.extend_from_slice(&0u32.to_le_bytes());
    let mut depths = BTreeMap::new();
    let mut scheduled = BTreeSet::new();
    let mut ids = BTreeSet::new();
    let mut pending = vec![(root, false)];
    while let Some((id, finish)) = pending.pop() {
        if depths.contains_key(&id) {
            continue;
        }
        let entry = ar.get(id).ok_or(Error::InvalidNode)?;
        if !finish {
            if scheduled.insert(id) && scheduled.len() > limits.max_nodes as usize {
                return Err(Error::NodeLimit);
            }
            pending.push((id, true));
            if let Data::Pair { left, right } = entry.inner {
                // Valid Reduction children precede parents. Reject corrupt/raw
                // arena nodes rather than loop on a cycle or forward pointer.
                if left >= id || right >= id {
                    return Err(Error::InvalidNode);
                }
                pending.push((right, false));
                pending.push((left, false));
            }
            continue;
        }
        let (payload, depth) = match entry.inner {
            Data::Atom { value } => (encode::encode_atom(value).to_vec(), 0),
            Data::Pair { left, right } => {
                let l = encode::particle_id(ar, left).ok_or(Error::InvalidNode)?;
                let r = encode::particle_id(ar, right).ok_or(Error::InvalidNode)?;
                particle(&l)?;
                particle(&r)?;
                let depth = bounded_depth(
                    *depths.get(&left).ok_or(Error::InvalidNode)?,
                    *depths.get(&right).ok_or(Error::InvalidNode)?,
                    limits,
                )?;
                (encode::encode_pair(&l, &r).to_vec(), depth)
            }
        };
        let pid = encode::particle_id(ar, id).ok_or(Error::InvalidNode)?;
        particle(&pid)?;
        if encode::particle_of(&payload).map_err(|_| Error::FieldRange)? != pid {
            return Err(Error::HashMismatch);
        }
        if !ids.insert(pid) {
            return Err(Error::Duplicate);
        }
        if out
            .len()
            .checked_add(33 + payload.len())
            .ok_or(Error::ByteLimit)?
            > limits.max_bytes
        {
            return Err(Error::ByteLimit);
        }
        out.extend_from_slice(&pid);
        out.push(payload.len() as u8);
        out.extend_from_slice(&payload);
        depths.insert(id, depth);
    }
    let count = u32::try_from(depths.len()).map_err(|_| Error::NodeLimit)?;
    out[40..44].copy_from_slice(&count.to_le_bytes());
    Ok(out)
}

struct Node {
    id: Particle,
    data: DecodedData,
    children: Option<(usize, usize)>,
    depth: u32,
}

fn parse(bytes: &[u8], limits: Limits) -> Result<Vec<Node>, Error> {
    if bytes.len() > limits.max_bytes {
        return Err(Error::ByteLimit);
    }
    if bytes.len() < HEADER {
        return Err(Error::Framing);
    }
    if &bytes[..8] != MAGIC {
        return Err(Error::Version);
    }
    let root = particle(&bytes[8..40])?;
    let count = u32::from_le_bytes(bytes[40..44].try_into().map_err(|_| Error::Framing)?);
    if count == 0 || count > limits.max_nodes {
        return Err(Error::NodeLimit);
    }
    if count as usize > (bytes.len() - HEADER) / MIN_ENTRY {
        return Err(Error::Framing);
    }
    let mut index = BTreeMap::new();
    let mut nodes: Vec<Node> = Vec::new();
    let mut pos = HEADER;
    for _ in 0..count {
        let head = bytes
            .get(pos..pos.checked_add(33).ok_or(Error::Framing)?)
            .ok_or(Error::Framing)?;
        let id = particle(&head[..32])?;
        if index.contains_key(&id) {
            return Err(Error::Duplicate);
        }
        let len = usize::from(head[32]);
        if len != 8 && len != 64 {
            return Err(Error::Framing);
        }
        pos += 33;
        let payload = bytes
            .get(pos..pos.checked_add(len).ok_or(Error::Framing)?)
            .ok_or(Error::Framing)?;
        pos += len;
        let data = encode::decode(payload).map_err(|_| Error::FieldRange)?;
        let (children, depth) = match &data {
            DecodedData::Atom(_) => (None, 0),
            DecodedData::Pair { left, right } => {
                particle(left)?;
                particle(right)?;
                let l: usize = *index.get(left).ok_or(Error::MissingReference)?;
                let r: usize = *index.get(right).ok_or(Error::MissingReference)?;
                (
                    Some((l, r)),
                    bounded_depth(nodes[l].depth, nodes[r].depth, limits)?,
                )
            }
        };
        if encode::particle_of(payload).map_err(|_| Error::FieldRange)? != id {
            return Err(Error::HashMismatch);
        }
        index.insert(id, nodes.len());
        nodes.push(Node {
            id,
            data,
            children,
            depth,
        });
    }
    if pos != bytes.len() {
        return Err(Error::Framing);
    }
    let root_index = *index.get(&root).ok_or(Error::MissingRoot)?;
    canonical_order(&nodes, root_index)?;
    Ok(nodes)
}

fn canonical_order(nodes: &[Node], root: usize) -> Result<(), Error> {
    let mut seen = vec![false; nodes.len()];
    let mut stack = vec![(root, false)];
    let mut next = 0;
    while let Some((i, finish)) = stack.pop() {
        if seen[i] {
            continue;
        }
        if finish {
            if i != next {
                return Err(Error::Order);
            }
            seen[i] = true;
            next += 1;
        } else {
            stack.push((i, true));
            if let Some((l, r)) = nodes[i].children {
                stack.push((r, false));
                stack.push((l, false));
            }
        }
    }
    if next != nodes.len() {
        return Err(Error::Order);
    }
    Ok(())
}

/// Validate completely before allocating VM nodes; allocation failures may leave
/// charged unreachable nodes, but never return a successful partial root.
pub fn decode<const N: usize>(
    ar: &mut Reduction<N>,
    bytes: &[u8],
    limits: Limits,
) -> Result<Order, Error> {
    let nodes = parse(bytes, limits)?;
    let mut orders = Vec::new();
    for node in nodes {
        let order = match node.data {
            DecodedData::Atom(value) => ar.atom(value),
            DecodedData::Pair { .. } => {
                let (l, r) = node.children.ok_or(Error::InvalidNode)?;
                ar.pair(orders[l], orders[r])
            }
        }
        .ok_or(Error::Allocation)?;
        if encode::particle_id(ar, order) != Some(node.id) {
            return Err(Error::HashMismatch);
        }
        orders.push(order);
    }
    orders.last().copied().ok_or(Error::MissingRoot)
}

#[cfg(test)]
#[path = "artifact/tests.rs"]
mod tests;
