# SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
FROM gcr.io/oss-fuzz-base/base-builder-rust
RUN git clone --depth 1 https://github.com/sebastienrousseau/metadata-gen metadata-gen
WORKDIR metadata-gen
COPY build.sh $SRC/
