use std.prelude.*;
use std.collections.treap;
(
    module:
    
    const KV = [K, V] type (
        .key :: K,
        .value :: V,
    );
    const t = [K, V] type (
        .inner :: treap.t[KV[K, V]],
    );
    const create = [K, V] () -> t[K, V] => (
        .inner = treap.create ()
    );
    
    const treap_split = [T] (v :: treap.t[T], f :: treap.node_splitter[T]) -> (treap.t[T], treap.t[T]) => (
        match v with (
            | :Empty => (:Empty, :Empty)
            | :Node node => match f &node with (
                | :RightSubtree => (
                    let left_left, left_right = treap_split (node.left, f);
                    let node = treap.update_data (
                        node,
                        .left = left_right,
                        .right = node.right,
                    );
                    left_left, node
                )
                | :LeftSubtree => (
                    let right_left, right_right = treap_split (node.right, f);
                    let node = treap.update_data (
                        node,
                        .left = node.left,
                        .right = right_left,
                    );
                    node, right_right
                )
                | :Node (left, right) => (
                    let left = treap.singleton left;
                    let right = treap.singleton right;
                    treap.join (node.left, left), treap.join (right, node.right)
                )
            )
        )
    );
    
    const add = [K, V] (map :: &t[K, V], key :: K, value :: V) => (
        get_or_init (map, key, () => value);
    );
    
    const get = [K, V] (map :: &t[K, V], key :: K) -> Option.t[type (&V)] => (
        let less, greater_or_equal = treap_split (
            map^.inner,
            data => (
                if data^.value.key < key then (
                    :LeftSubtree
                ) else (
                    :RightSubtree
                )
            ),
        );
        let equal, greater = treap_split (
            greater_or_equal,
            data => (
                if data^.value.key <= key then (
                    :LeftSubtree
                ) else (
                    :RightSubtree
                )
            ),
        );
        if treap.length &equal == 0 then (
            :None
        ) else (
            :Some (&(treap.at (&equal, 0))^.value)
        )
    );
    
    const get_or_init = [K, V] (
        map :: &t[K, V],
        key :: K,
        init :: () -> V,
    ) -> &V => (
        let less, greater_or_equal = treap_split (
            map^.inner,
            data => (
                if data^.value.key < key then (
                    :LeftSubtree
                ) else (
                    :RightSubtree
                )
            ),
        );
        let equal, greater = treap_split (
            greater_or_equal,
            data => (
                if data^.value.key <= key then (
                    :LeftSubtree
                ) else (
                    :RightSubtree
                )
            ),
        );
        if treap.length &equal == 0 then (
            equal = treap.singleton (.key, .value = init ());
        );
        map^.inner = treap.join (less, treap.join (equal, greater));
        &(treap.at (&equal, 0))^.value
    );
    
    const iter = [K, V] (map :: &t[K, V], f :: &KV[K, V] -> ()) => (
        treap.iter (&map^.inner, f)
    );
)
