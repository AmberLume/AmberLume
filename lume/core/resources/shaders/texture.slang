int2 texture_size<T : ITexelElement>(Texture2D<T> texture) {
    uint width;
    uint height;
    texture.GetDimensions(width, height);

    return int2(width, height);
}

int2 texture_size<T : ITexelElement>(RWTexture2D<T> texture) {
    uint width;
    uint height;
    texture.GetDimensions(width, height);

    return int2(width, height);
}
