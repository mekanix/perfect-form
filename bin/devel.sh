#!/bin/sh

BINDIR=$(dirname $0)

cd "${BINDIR}/.."
topcoat dev --bin perfect-form
