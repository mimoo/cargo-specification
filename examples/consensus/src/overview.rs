//~ The consensus specification describes the mechanism used by the Diem Payment Network (LPN) validators to agree on both ordering and transaction execution output under byzantine fault-tolerant (BFT) conditions - at most f validators (where f < (all validators)/3) are faulty or malicious. Currently, the consensus specification is an implementation of DiemBFT, a BFT consensus protocol that ensures liveness and safety under partial synchrony. The DiemBFT whitepaper describes a high level overview of the protocol, the liveness and safety proofs, and a rationale on why DiemBFT was adopted for the LPN. This document specifies how to implement the DiemBFT protocol in order to participate as a validating node in the LPN.
//~
//~ This document is organized as follows:
//~ Architecture - the components of the specification and how they interact.
//~ Data structures - common data structures that are part of this specification.
//~ Network messages - consensus messages that are sent across the wire to other validators.
//~ Abstracted modules - The components this specification depends on.
//~ Consensus modules - The components built upon common data structures that are described as a part of this specification.
//~ All network communication occurs over DiemNet and any serialization, deserialization and hashing is determined by BCS.
