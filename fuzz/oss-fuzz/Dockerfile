# SPDX-FileCopyrightText: 2024 - 2026 metadata-gen contributors
# SPDX-License-Identifier: Apache-2.0 OR MIT
FROM gcr.io/oss-fuzz-base/base-builder-rust:latest@sha256:d345218878c78feed410f0ad57268617a368912b224f844bbdb71c54c0b9955a
RUN git clone --depth 1 https://github.com/sebastienrousseau/metadata-gen metadata-gen
WORKDIR metadata-gen
COPY build.sh $SRC/
