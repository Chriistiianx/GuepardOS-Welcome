#!/bin/sh

mkdir -p "${DESTDIR}/${MESON_INSTALL_PREFIX}/share/guepardos-welcome/"
cp -r "${MESON_SOURCE_ROOT}/src/scripts" "${DESTDIR}/${MESON_INSTALL_PREFIX}/share/guepardos-welcome/"
