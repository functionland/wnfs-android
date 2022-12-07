use std::rc::Rc;

use anyhow::{bail, Result};
use skip_ratchet::{ratchet::PreviousIterator, Ratchet};

use super::{PrivateDirectory, PrivateFile, PrivateForest, PrivateNode, PrivateNodeHeader};

use crate::{BlockStore, FsError, PathNodes, PathNodesResult};

//--------------------------------------------------------------------------------------------------
// Type Definitions
//--------------------------------------------------------------------------------------------------

/// Represents the state of an iterator through the history
/// of a private node on a path relative to a root directory.
pub struct PrivateNodeOnPathHistory {
    /// Keep a reference to the version of the forest used upon construction.
    /// It could *technically* change what's behind a certain key in between
    /// previous node requests, this forces it to be consistent.
    forest: Rc<PrivateForest>,
    /// Keep the original discrepancy budget for consistency & ease of use.
    discrepancy_budget: usize,
    /// The history of each path segment leading up to the final node
    path: Vec<PathSegmentHistory>,
    /// The target node's history
    target: PrivateNodeHistory,
}

struct PathSegmentHistory {
    /// The directory that the history was originally created relative to.
    dir: Rc<PrivateDirectory>,
    /// The history of said directory.
    history: PrivateNodeHistory,
    /// The name of the child node to follow for history next.
    path_segment: String,
}

/// This represents the state of an iterator through the history of
/// only a single private node. It can only be constructed when you
/// know the past ratchet state of such a node.
pub struct PrivateNodeHistory {
    /// Keep a reference to the version of the forest used upon construction.
    /// It could *technically* change what's behind a certain key in between
    /// previous node requests, this forces it to be consistent.
    forest: Rc<PrivateForest>,
    /// The private node header is all we need to look up private nodes in the forest.
    /// This will always be the header of the *next* version after what's retrieved from
    /// the `ratchets` iterator.
    header: PrivateNodeHeader,
    /// The iterator for previous revision ratchets.
    ratchets: PreviousIterator,
}

impl PrivateNodeHistory {
    /// Create a history iterator for given private node up until `past_ratchet`.
    ///
    /// There must be an `n > 0` for which `node.get_header().ratchet == past_ratchet.inc_by(n)`.
    ///
    /// Discrepancy budget is used to bound the search for the actual `n`
    /// and prevent infinite looping in case it doesn't exist.
    pub fn of(
        node: &PrivateNode,
        past_ratchet: &Ratchet,
        discrepancy_budget: usize,
        forest: Rc<PrivateForest>,
    ) -> Result<Self> {
        Self::from_header(
            node.get_header().clone(),
            past_ratchet,
            discrepancy_budget,
            forest,
        )
    }

    /// Create a history iterator for a node given its header.
    ///
    /// See also `PrivateNodeHistory::of`.
    pub fn from_header(
        header: PrivateNodeHeader,
        past_ratchet: &Ratchet,
        discrepancy_budget: usize,
        forest: Rc<PrivateForest>,
    ) -> Result<Self> {
        let forest = Rc::clone(&forest);
        let ratchets = PreviousIterator::new(past_ratchet, &header.ratchet, discrepancy_budget)
            .map_err(FsError::NoIntermediateRatchet)?;
        Ok(PrivateNodeHistory {
            forest,
            header,
            ratchets,
        })
    }

    /// Step the history one step back and retrieve the private node at the
    /// previous point in history.
    ///
    /// Returns `None` if there is no such node in the `PrivateForest` at that point in time.
    pub async fn get_previous_node<B: BlockStore>(
        &mut self,
        store: &B,
    ) -> Result<Option<PrivateNode>> {
        match self.ratchets.next() {
            None => Ok(None),
            Some(previous_ratchet) => {
                // TODO(matheus23) Make the `resolve_bias` be biased towards eventual `previous` backpointers.
                self.header.ratchet = previous_ratchet;
                self.forest
                    .get(
                        &self.header.get_private_ref(),
                        PrivateForest::resolve_lowest,
                        store,
                    )
                    .await
            }
        }
    }

    /// Like `previous_node`, but attempts to resolve a directory.
    ///
    /// Returns `None` if there is no previous node with that revision in the `PrivateForest`,
    /// throws `FsError::NotADirectory` if the previous node happens to not be a directory.
    /// That should only happen for all nodes or for none.
    pub async fn get_previous_dir<B: BlockStore>(
        &mut self,
        store: &B,
    ) -> Result<Option<Rc<PrivateDirectory>>> {
        match self.get_previous_node(store).await? {
            Some(PrivateNode::Dir(dir)) => Ok(Some(dir)),
            Some(_) => Err(FsError::NotADirectory.into()),
            None => Ok(None),
        }
    }

    /// Like `previous_node`, but attempts to resolve a file.
    ///
    /// Returns `None` if there is no previous node with that revision in the `PrivateForest`,
    /// throws `FsError::NotAFile` if the previous node happens to not be a file.
    /// That should only happen for all nodes or for none.
    pub async fn get_previous_file<B: BlockStore>(
        &mut self,
        store: &B,
    ) -> Result<Option<Rc<PrivateFile>>> {
        match self.get_previous_node(store).await? {
            Some(PrivateNode::File(file)) => Ok(Some(file)),
            Some(_) => Err(FsError::NotAFile.into()),
            None => Ok(None),
        }
    }
}

impl PrivateNodeOnPathHistory {
    /// Construct a history iterator for a private node at some path relative
    /// to some root directory.
    ///
    /// Returns errors when there is no private node at given path,
    /// or if the given `past_ratchet` is not within the `discrepancy_budget` to
    /// the given root `directory`, or simply unrelated.
    ///
    /// When `search_latest` is true, it follow the path in the current revision
    /// down to the child, and then look for the latest revision of the target node,
    /// including all in-between versions in the history.
    pub async fn of<B: BlockStore>(
        directory: Rc<PrivateDirectory>,
        past_ratchet: &Ratchet,
        discrepancy_budget: usize,
        path_segments: &[String],
        search_latest: bool,
        forest: Rc<PrivateForest>,
        store: &B,
    ) -> Result<PrivateNodeOnPathHistory> {
        // To get the history on a node on a path from a given directory that we
        // know its newest and oldest ratchet of, we need to generate
        // `PrivateNodeHistory`s for each path segment up to the last node.
        //
        // This is what this function is doing, it constructs the `PrivateNodeOnPathHistory`.
        //
        // Stepping that history forward is then done in `PrivateNodeOnPathHistory#previous`.

        let (target_path, path_segments) = match path_segments.split_last() {
            None => {
                return Ok(PrivateNodeOnPathHistory {
                    forest: Rc::clone(&forest),
                    discrepancy_budget,
                    path: Vec::with_capacity(0),
                    target: PrivateNodeHistory::of(
                        &PrivateNode::Dir(directory),
                        past_ratchet,
                        discrepancy_budget,
                        Rc::clone(&forest),
                    )?,
                });
            }
            Some(split) => split,
        };

        let (path, target_history) = Self::path_nodes_and_target_history(
            Rc::clone(&directory),
            discrepancy_budget,
            path_segments,
            target_path,
            search_latest,
            Rc::clone(&forest),
            store,
        )
        .await?;

        let path =
            Self::path_segment_empty_histories(path, Rc::clone(&forest), discrepancy_budget)?;

        let mut previous_iter = PrivateNodeOnPathHistory {
            forest: Rc::clone(&forest),
            discrepancy_budget,
            path,
            target: target_history,
        };

        // For the first part of the path, we specifically set the history ourselves,
        // because we've had `past_ratchet` passed in from the outside.

        let new_ratchet = directory.header.ratchet.clone();

        previous_iter.path[0].history.ratchets =
            PreviousIterator::new(past_ratchet, &new_ratchet, discrepancy_budget)
                .map_err(FsError::NoIntermediateRatchet)?;

        Ok(previous_iter)
    }

    /// Accumulates the path nodes towards a target node and
    /// creates a `PrivateNodeHistory` for that target node from
    /// the latest version it could find (if `search_latest` is true)
    /// until the current revision.
    ///
    /// If `search_latest` is false, the target history is empty.
    async fn path_nodes_and_target_history<B: BlockStore>(
        dir: Rc<PrivateDirectory>,
        discrepancy_budget: usize,
        path_segments: &[String],
        target_path_segment: &String,
        search_latest: bool,
        forest: Rc<PrivateForest>,
        store: &B,
    ) -> Result<(Vec<(Rc<PrivateDirectory>, String)>, PrivateNodeHistory)> {
        // We only search for the latest revision in the private node.
        // It may have been deleted in future versions of its ancestor directories.
        let path_nodes = match dir
            .get_path_nodes(path_segments, false, &forest, store)
            .await?
        {
            PathNodesResult::Complete(path_nodes) => path_nodes,
            PathNodesResult::MissingLink(_, _) => bail!(FsError::NotFound),
            PathNodesResult::NotADirectory(_, _) => bail!(FsError::NotADirectory),
        };

        // TODO(matheus23) refactor using let-else once rust stable 1.65 released (Nov 3rd)
        let target = match (*path_nodes.tail)
            .lookup_node(target_path_segment, false, &forest, store)
            .await?
        {
            Some(target) => target,
            None => bail!(FsError::NotFound),
        };

        let target_latest = if search_latest {
            target.search_latest(&forest, store).await?
        } else {
            target.clone()
        };

        let target_history = PrivateNodeHistory::of(
            &target_latest,
            &target.get_header().ratchet,
            discrepancy_budget,
            Rc::clone(&forest),
        )?;

        let PathNodes { mut path, tail } = path_nodes;

        path.push((tail, target_path_segment.to_string()));

        Ok((path, target_history))
    }

    /// Takes a path of directories and initializes each path segment with an empty history.
    fn path_segment_empty_histories(
        path: Vec<(Rc<PrivateDirectory>, String)>,
        forest: Rc<PrivateForest>,
        discrepancy_budget: usize,
    ) -> Result<Vec<PathSegmentHistory>> {
        let mut segments = Vec::new();
        for (dir, path_segment) in path {
            segments.push(PathSegmentHistory {
                dir: Rc::clone(&dir),
                history: PrivateNodeHistory::of(
                    &PrivateNode::Dir(Rc::clone(&dir)),
                    &dir.header.ratchet,
                    discrepancy_budget,
                    Rc::clone(&forest),
                )?,
                path_segment,
            });
        }

        Ok(segments)
    }

    /// Step the history one revision back and retrieve the node at the configured path.
    ///
    /// Returns `None` if there is no more previous revisions.
    pub async fn get_previous<B: BlockStore>(&mut self, store: &B) -> Result<Option<PrivateNode>> {
        // Finding the previous revision of a node works by trying to get
        // the previous revision of the path elements starting on the deepest
        // path node working upwards, in case the history of lower nodes
        // have been exhausted.
        //
        // Once another history entry on the path has been found, we proceed
        // to work back trying to construct new history entries by going downwards
        // on the same path from an older root revision, until we've completed
        // the whole path and found new history entries in every segment.

        if let Some(node) = self.target.get_previous_node(store).await? {
            return Ok(Some(node));
        }

        // TODO(matheus23) refactor using let-else once rust stable 1.65 released (Nov 3rd)
        let working_stack = match self.find_and_step_segment_history(store).await? {
            Some(stack) => stack,
            None => return Ok(None),
        };

        if !self
            .repopulate_segment_histories(working_stack, store)
            .await?
        {
            return Ok(None);
        }

        let ancestor = self.path.last().expect(
            "Should not happen: path stack was empty after call to repopulate_segment_histories",
        );

        // TODO(matheus23) refactor using let-else once rust stable 1.65 released (Nov 3rd)
        let older_node = match ancestor
            .dir
            .lookup_node(&ancestor.path_segment, false, &self.forest, store)
            .await?
        {
            Some(older_node) => older_node,
            None => return Ok(None),
        };

        self.target = match PrivateNodeHistory::from_header(
            self.target.header.clone(),
            &older_node.get_header().ratchet,
            self.discrepancy_budget,
            Rc::clone(&self.forest),
        ) {
            Ok(history) => history,
            // NoIntermediateRatchet error
            Err(_) => {
                // The target element lives at the same path but has ratchets that are further
                // apart than `discrepancy_budget`.
                // It's likely this node was deleted and recreated in between these history
                // steps. Or it had its key rotated. Either way, the history stops here.
                return Ok(None);
            }
        };

        self.target.get_previous_node(store).await
    }

    /// Pops off elements from the path segment history stack until a
    /// path segment history is found which has history entries.
    /// Then this will put the previous directory on that stack and return
    /// all elements that were popped off.
    ///
    /// Returns None if the no path segment history in the stack has any
    /// more history entries.
    async fn find_and_step_segment_history<B: BlockStore>(
        &mut self,
        store: &B,
    ) -> Result<Option<Vec<(Rc<PrivateDirectory>, String)>>> {
        let mut working_stack = Vec::with_capacity(self.path.len());

        loop {
            // Pop elements off the end of the path
            if let Some(mut segment) = self.path.pop() {
                // Try to find a path segment for which we have previous history entries
                if let Some(prev) = segment.history.get_previous_dir(store).await? {
                    segment.dir = prev;
                    self.path.push(segment);
                    // Once found, we can continue.
                    break;
                }

                working_stack.push((segment.dir, segment.path_segment));
            } else {
                // We have exhausted all histories of all path segments.
                // There's no way we can produce more history entries.
                return Ok(None);
            }
        }

        Ok(Some(working_stack))
    }

    /// After having popped off elements from the path segment history stack,
    /// and leaving behind a history-steppable element,
    /// push back steppable histories onto the stack.
    ///
    /// Must only be called when the path segment history stack has
    /// a steppable history entry on top.
    ///
    /// Returns false if there's no corresponding path in the previous revision.
    async fn repopulate_segment_histories<B: BlockStore>(
        &mut self,
        working_stack: Vec<(Rc<PrivateDirectory>, String)>,
        store: &B,
    ) -> Result<bool> {
        // Work downwards from the previous history entry of a path segment we found
        for (directory, path_segment) in working_stack {
            let ancestor = self
                .path
                .last()
                .expect("Should not happen: repopulate_segment_histories called when the path stack was empty.");

            // Go down from the older ancestor directory parallel to the new revision's path
            // TODO(matheus23) refactor using let-else once rust stable 1.65 released (Nov 3rd)
            let older_directory = match ancestor
                .dir
                .lookup_node(&ancestor.path_segment, false, &self.forest, store)
                .await?
            {
                Some(PrivateNode::Dir(older_directory)) => older_directory,
                _ => return Ok(false),
            };

            let mut directory_history = match PrivateNodeHistory::of(
                &PrivateNode::Dir(directory),
                &older_directory.header.ratchet,
                self.discrepancy_budget,
                Rc::clone(&self.forest),
            ) {
                Ok(history) => history,
                // NoIntermediateRatchet error
                Err(_) => {
                    // in this case the two directories share the same name in different revisions,
                    // but their keys aren't related. It's likely that they don't share identity
                    // or there's some key rotation in between meaning we can't follow their history.
                    return Ok(false);
                }
            };

            // We need to find the in-between history entry! See the test case `previous_with_multiple_child_changes`.
            // TODO(matheus23) refactor using let-else once rust stable 1.65 released (Nov 3rd)
            let directory_prev = match directory_history.get_previous_dir(store).await? {
                Some(dir) => dir,
                _ => return Ok(false),
            };

            self.path.push(PathSegmentHistory {
                dir: directory_prev,
                history: directory_history,
                path_segment,
            });
        }

        Ok(true)
    }
}

//--------------------------------------------------------------------------------------------------
// Tests
//--------------------------------------------------------------------------------------------------

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        private::{namefilter::Namefilter, PrivateDirectory, PrivateOpResult},
        MemoryBlockStore,
    };
    use chrono::Utc;
    use proptest::test_runner::{RngAlgorithm, TestRng};

    struct TestSetup {
        rng: TestRng,
        store: MemoryBlockStore,
        forest: Rc<PrivateForest>,
        root_dir: Rc<PrivateDirectory>,
        discrepancy_budget: usize,
    }

    impl TestSetup {
        fn new() -> Self {
            let mut rng = TestRng::deterministic_rng(RngAlgorithm::ChaCha);
            let store = MemoryBlockStore::default();
            let root_dir = Rc::new(PrivateDirectory::new(
                Namefilter::default(),
                Utc::now(),
                &mut rng,
            ));

            Self {
                rng,
                store,
                forest: Rc::new(PrivateForest::new()),
                root_dir,
                discrepancy_budget: 1_000_000,
            }
        }
    }

    #[async_std::test]
    async fn previous_of_root_node() {
        let TestSetup {
            mut rng,
            mut store,
            forest,
            root_dir,
            discrepancy_budget,
        } = TestSetup::new();

        let rng = &mut rng;
        let store = &mut store;

        let forest = forest
            .put(
                root_dir.header.get_saturated_name(),
                &root_dir.header.get_private_ref(),
                &PrivateNode::Dir(Rc::clone(&root_dir)),
                store,
                rng,
            )
            .await
            .unwrap();

        let past_ratchet = root_dir.header.ratchet.clone();

        let PrivateOpResult {
            root_dir, forest, ..
        } = root_dir
            .write(
                &["file.txt".into()],
                true,
                Utc::now(),
                b"file".to_vec(),
                forest,
                store,
                rng,
            )
            .await
            .unwrap();

        let PrivateOpResult {
            root_dir, forest, ..
        } = root_dir
            .mkdir(&["docs".into()], true, Utc::now(), forest, store, rng)
            .await
            .unwrap();

        let mut iterator = PrivateNodeOnPathHistory::of(
            root_dir,
            &past_ratchet,
            discrepancy_budget,
            &[],
            true,
            Rc::clone(&forest),
            store,
        )
        .await
        .unwrap();

        assert!(iterator.get_previous(store).await.unwrap().is_some());
        assert!(iterator.get_previous(store).await.unwrap().is_some());
        assert!(iterator.get_previous(store).await.unwrap().is_none());
    }

    /// This test will generate the following file system structure:
    ///
    /// (horizontal = time series, vertical = hierarchy)
    /// ```plain
    /// ┌────────────┐              ┌────────────┐              ┌────────────┐
    /// │            │              │            │              │            │
    /// │    Root    ├─────────────►│    Root    ├─────────────►│    Root    │
    /// │            │              │            │              │            │
    /// └────────────┘              └─────┬──────┘              └─────┬──────┘
    ///                                   │                           │
    ///                                   │                           │
    ///                                   ▼                           ▼
    ///                             ┌────────────┐              ┌────────────┐
    ///                             │            │              │            │
    ///                             │    Docs    ├─────────────►│    Docs    │
    ///                             │            │              │            │
    ///                             └─────┬──────┘              └─────┬──────┘
    ///                                   │                           │
    ///                                   │                           │
    ///                                   ▼                           ▼
    ///                             ┌────────────┐              ┌────────────┐
    ///                             │            │              │            │
    ///                             │  Notes.md  ├─────────────►│  Notes.md  │
    ///                             │            │              │            │
    ///                             └────────────┘              └────────────┘
    /// ```
    ///
    /// Then, given the skip ratchet for revision 0 of "Root" and revision 2 of "Root",
    /// it will ask for the backwards-history of the "Root/Docs/Notes.md" file.
    #[async_std::test]
    async fn previous_of_path() {
        let TestSetup {
            mut rng,
            mut store,
            forest,
            root_dir,
            discrepancy_budget,
        } = TestSetup::new();

        let rng = &mut rng;
        let store = &mut store;

        let forest = forest
            .put(
                root_dir.header.get_saturated_name(),
                &root_dir.header.get_private_ref(),
                &PrivateNode::Dir(Rc::clone(&root_dir)),
                store,
                rng,
            )
            .await
            .unwrap();

        let past_ratchet = root_dir.header.ratchet.clone();

        let path = ["Docs".into(), "Notes.md".into()];

        let PrivateOpResult {
            root_dir, forest, ..
        } = root_dir
            .write(&path, true, Utc::now(), b"Hi".to_vec(), forest, store, rng)
            .await
            .unwrap();

        let PrivateOpResult {
            root_dir, forest, ..
        } = root_dir
            .write(
                &path,
                true,
                Utc::now(),
                b"World".to_vec(),
                forest,
                store,
                rng,
            )
            .await
            .unwrap();

        let mut iterator = PrivateNodeOnPathHistory::of(
            root_dir,
            &past_ratchet,
            discrepancy_budget,
            &path,
            true,
            Rc::clone(&forest),
            store,
        )
        .await
        .unwrap();

        assert_eq!(
            iterator
                .get_previous(store)
                .await
                .unwrap()
                .unwrap()
                .as_file()
                .unwrap()
                .get_content(&forest, store)
                .await
                .unwrap(),
            b"Hi".to_vec()
        );

        assert!(iterator.get_previous(store).await.unwrap().is_none());
    }

    /// This test will generate the following file system structure:
    ///
    /// (horizontal = time series, vertical = hierarchy)
    /// ```plain
    /// ┌────────────┐              ┌────────────┐
    /// │            │              │            │
    /// │    Root    ├─────────────►│    Root    │
    /// │            │              │            │
    /// └────────────┘              └─────┬──────┘
    ///                                   │
    ///                                   │
    ///                                   ▼
    ///                             ┌────────────┐              ┌────────────┐
    ///                             │            │              │            │
    ///                             │    Docs    ├─────────────►│    Docs    │
    ///                             │            │              │            │
    ///                             └─────┬──────┘              └─────┬──────┘
    ///                                   │                           │
    ///                                   │                           │
    ///                                   ▼                           ▼
    ///                             ┌────────────┐              ┌────────────┐
    ///                             │            │              │            │
    ///                             │  Notes.md  ├─────────────►│  Notes.md  │
    ///                             │            │              │            │
    ///                             └────────────┘              └────────────┘
    /// ```
    ///
    /// This is testing a case where the file system wasn't rooted completely.
    /// Imagine someone wrote the `Notes.md` file with only access up to `Root/Docs`.
    /// The file system diagram looks like this:
    #[async_std::test]
    async fn previous_of_seeking() {
        let TestSetup {
            mut rng,
            mut store,
            forest,
            root_dir,
            discrepancy_budget,
        } = TestSetup::new();

        let rng = &mut rng;
        let store = &mut store;

        let forest = forest
            .put(
                root_dir.header.get_saturated_name(),
                &root_dir.header.get_private_ref(),
                &PrivateNode::Dir(Rc::clone(&root_dir)),
                store,
                rng,
            )
            .await
            .unwrap();

        let past_ratchet = root_dir.header.ratchet.clone();

        let path = ["Docs".into(), "Notes.md".into()];

        let PrivateOpResult {
            root_dir, forest, ..
        } = root_dir
            .write(&path, true, Utc::now(), b"Hi".to_vec(), forest, store, rng)
            .await
            .unwrap();

        let PrivateOpResult {
            root_dir,
            forest,
            result: docs_dir,
            ..
        } = root_dir
            .get_node(&["Docs".into()], true, forest, store)
            .await
            .unwrap();

        let docs_dir = docs_dir.unwrap().as_dir().unwrap();

        let PrivateOpResult { forest, .. } = docs_dir
            .write(
                &["Notes.md".into()],
                true,
                Utc::now(),
                b"World".to_vec(),
                forest,
                store,
                rng,
            )
            .await
            .unwrap();

        let mut iterator = PrivateNodeOnPathHistory::of(
            root_dir,
            &past_ratchet,
            discrepancy_budget,
            &path,
            true,
            Rc::clone(&forest),
            store,
        )
        .await
        .unwrap();

        assert_eq!(
            iterator
                .get_previous(store)
                .await
                .unwrap()
                .unwrap()
                .as_file()
                .unwrap()
                .get_content(&forest, store)
                .await
                .unwrap(),
            b"Hi".to_vec()
        );

        assert!(iterator.get_previous(store).await.unwrap().is_none());
    }

    /// This test will generate the following file system structure:
    ///
    /// (horizontal = time series, vertical = hierarchy)
    /// ```plain
    /// ┌────────────┐                              ┌────────────┐
    /// │            │                              │            │
    /// │    Root    ├─────────────────────────────►│    Root    │
    /// │            │                              │            │
    /// └─────┬──────┘                              └─────┬──────┘
    ///       │                                           │
    ///       │                                           │
    ///       ▼                                           ▼
    /// ┌────────────┐        ┌────────────┐        ┌────────────┐
    /// │            │        │            │        │            │
    /// │    Docs    ├───────►│    Docs    ├───────►│    Docs    │
    /// │            │        │            │        │            │
    /// └─────┬──────┘        └─────┬──────┘        └─────┬──────┘
    ///       │                     │                     │
    ///       │                     │                     │
    ///       ▼                     ▼                     ▼
    /// ┌────────────┐        ┌────────────┐        ┌────────────┐
    /// │            │        │            │        │            │
    /// │  Notes.md  ├───────►│  Notes.md  ├───────►│  Notes.md  │
    /// │            │        │            │        │            │
    /// └────────────┘        └────────────┘        └────────────┘
    /// ```
    ///
    /// This case happens when someone who only has access up to
    /// `Root/Docs` writes two revisions of `Notes.md` and
    /// is later rooted by another peer that has full root access.
    #[async_std::test]
    async fn previous_with_multiple_child_changes() {
        let TestSetup {
            mut rng,
            mut store,
            forest,
            root_dir,
            discrepancy_budget,
        } = TestSetup::new();

        let rng = &mut rng;
        let store = &mut store;

        let path = ["Docs".into(), "Notes.md".into()];

        let PrivateOpResult {
            root_dir, forest, ..
        } = root_dir
            .write(
                &path,
                true,
                Utc::now(),
                b"rev 0".to_vec(),
                forest,
                store,
                rng,
            )
            .await
            .unwrap();

        let past_ratchet = root_dir.header.ratchet.clone();

        let PrivateOpResult {
            root_dir,
            forest,
            result: docs_dir,
            ..
        } = root_dir
            .get_node(&["Docs".into()], true, forest, store)
            .await
            .unwrap();

        let docs_dir = docs_dir.unwrap().as_dir().unwrap();

        let PrivateOpResult { forest, .. } = docs_dir
            .write(
                &["Notes.md".into()],
                true,
                Utc::now(),
                b"rev 1".to_vec(),
                forest,
                store,
                rng,
            )
            .await
            .unwrap();

        let PrivateOpResult {
            root_dir, forest, ..
        } = root_dir
            .write(
                &path,
                true,
                Utc::now(),
                b"rev 2".to_vec(),
                forest,
                store,
                rng,
            )
            .await
            .unwrap();

        let mut iterator = PrivateNodeOnPathHistory::of(
            root_dir,
            &past_ratchet,
            discrepancy_budget,
            &path,
            true,
            Rc::clone(&forest),
            store,
        )
        .await
        .unwrap();

        assert_eq!(
            iterator
                .get_previous(store)
                .await
                .unwrap()
                .unwrap()
                .as_file()
                .unwrap()
                .get_content(&forest, store)
                .await
                .unwrap(),
            b"rev 1".to_vec()
        );

        assert_eq!(
            iterator
                .get_previous(store)
                .await
                .unwrap()
                .unwrap()
                .as_file()
                .unwrap()
                .get_content(&forest, store)
                .await
                .unwrap(),
            b"rev 0".to_vec()
        );

        assert!(iterator.get_previous(store).await.unwrap().is_none());
    }

    /// This test will generate the following file system structure:
    ///
    /// (horizontal = time series, vertical = hierarchy)
    /// ```plain
    /// ┌────────────┐    ┌────────────┐    ┌────────────┐
    /// │            │    │            │    │            │
    /// │    Root    ├───►│    Root    ├───►│    Root    │
    /// │            │    │            │    │            │
    /// └─────┬──────┘    └─────┬──────┘    └─────┬──────┘
    ///       │                 │                 │
    ///       │ ┌───────────────┘                 │
    ///       ▼ ▼                                 ▼
    /// ┌────────────┐                      ┌────────────┐
    /// │            │                      │            │
    /// │    Docs    ├─────────────────────►│    Docs    │
    /// │            │                      │            │
    /// └─────┬──────┘                      └─────┬──────┘
    ///       │                                   │
    ///       │                                   │
    ///       ▼                                   ▼
    /// ┌────────────┐                      ┌────────────┐
    /// │            │                      │            │
    /// │  Notes.md  ├─────────────────────►│  Notes.md  │
    /// │            │                      │            │
    /// └────────────┘                      └────────────┘
    /// ```
    ///
    /// This scenario may happen very commonly when things are
    /// written to the root directory that aren't related to
    /// the path that is looked at for its history.
    #[async_std::test]
    async fn previous_with_unrelated_changes() {
        let TestSetup {
            mut rng,
            mut store,
            forest,
            root_dir,
            discrepancy_budget,
        } = TestSetup::new();

        let rng = &mut rng;
        let store = &mut store;

        let path = ["Docs".into(), "Notes.md".into()];

        let PrivateOpResult {
            root_dir, forest, ..
        } = root_dir
            .write(
                &path,
                true,
                Utc::now(),
                b"rev 0".to_vec(),
                forest,
                store,
                rng,
            )
            .await
            .unwrap();

        let past_ratchet = root_dir.header.ratchet.clone();

        let root_dir = {
            let mut tmp = (*root_dir).clone();
            tmp.advance_ratchet();
            Rc::new(tmp)
        };

        let forest = forest
            .put(
                root_dir.header.get_saturated_name(),
                &root_dir.header.get_private_ref(),
                &PrivateNode::Dir(Rc::clone(&root_dir)),
                store,
                rng,
            )
            .await
            .unwrap();

        let PrivateOpResult {
            root_dir, forest, ..
        } = root_dir
            .write(
                &path,
                true,
                Utc::now(),
                b"rev 1".to_vec(),
                forest,
                store,
                rng,
            )
            .await
            .unwrap();

        assert_eq!(
            root_dir.header.ratchet.compare(&past_ratchet, 100).unwrap(),
            2
        );

        let mut iterator = PrivateNodeOnPathHistory::of(
            root_dir,
            &past_ratchet,
            discrepancy_budget,
            &path,
            true,
            Rc::clone(&forest),
            store,
        )
        .await
        .unwrap();

        assert_eq!(
            iterator
                .get_previous(store)
                .await
                .unwrap()
                .unwrap()
                .as_file()
                .unwrap()
                .get_content(&forest, store)
                .await
                .unwrap(),
            b"rev 0".to_vec()
        );

        assert!(iterator.get_previous(store).await.unwrap().is_none());
    }
}
