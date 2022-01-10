pub use crate::leases::{Lease, Leases};
use crate::leases::{LeasesMethods};


pub struct LeasesFilter {}

/// Filter leases matching criteria from a Leases
impl LeasesFilter {
    /// Returns a copy of all leases for the client with `mac`. The `mac` pattern can be a whole
    /// mac address, or the first n characters of a mac. The filter uses .starts_with to compare.
    pub fn by_mac_all(leases: &Leases, mac: &str) -> Vec<Lease> {
        let mut new_leases = Vec::new();

        for ndx in 0..leases.count() {
            if let Some(hw) = leases[ndx].hardware.as_ref() {
                if hw.mac.starts_with(&mac) {
                    new_leases.push(leases[ndx].clone());
                }
            }
        }

        new_leases
    }

    /// Returns a copy of all currently-active leases for the client
    pub fn by_mac_active(leases: &Leases, mac: &str) -> Vec<Lease> {
        let leases = LeasesFilter::by_mac_all(&leases, &mac);
        let mut active : Vec<Lease> = Vec::new();

        for lease in leases.iter() {
            if lease.is_active() {
                active.push(lease.clone());
            }
        }

        active
    }
}

pub struct LeaseFilterBuilder {
    leases: Vec<Lease>,
    match_indexes: Vec<usize>,
}

impl LeaseFilterBuilder {
    pub fn new(leases: &Leases) -> LeaseFilterBuilder {
        let mut leases_vec : Vec<Lease> = Vec::new();
        for ndx in 0..leases.count() {
            leases_vec.push(leases[ndx].clone())
        }

        LeaseFilterBuilder {leases: leases_vec, match_indexes: (0..leases.count()).collect()}
    }

    pub fn on_mac(&mut self, mac: &str) -> &mut Self {
        let mut keep_ndx : Vec<usize> = Vec::new();

        for ndx in self.match_indexes.iter() {
            let lease = &self.leases[*ndx];

            if lease.client().starts_with(&mac) {
                keep_ndx.push(ndx.clone());
            }
        }

        self.match_indexes.retain(|&i| keep_ndx.contains(&i));

        self
    }

    pub fn on_active(&mut self) -> &mut Self {
        let mut keep_ndx : Vec<usize> = Vec::new();

        for ndx in self.match_indexes.iter() {
            let lease = &self.leases[*ndx];

            if lease.is_active() {
                keep_ndx.push(ndx.clone());
            }
        }

        self.match_indexes.retain(|&i| keep_ndx.contains(&i));

        self
    }

    pub fn collect(&mut self) -> Leases {
        let mut leases = Leases::new();

        for ndx in self.match_indexes.iter() {
            leases.push(self.leases[*ndx].clone())
        }

        leases
    }
}