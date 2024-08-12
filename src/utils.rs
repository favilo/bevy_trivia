pub type BiHashMap<K, V> = bimap::BiHashMap<
    K,
    V,
    bevy::utils::hashbrown::hash_map::DefaultHashBuilder,
    bevy::utils::hashbrown::hash_map::DefaultHashBuilder,
>;
