"""Garantir colunas de metadados e versionamento em saves via Alembic

Revision ID: f1a2b3c4d5e6
Revises: e5b926fb5c96
Create Date: 2026-10-28 15:55:00.000000

"""

import sqlalchemy as sa
from alembic import op

# revision identifiers, used by Alembic.
revision = "f1a2b3c4d5e6"
down_revision = "e5b926fb5c96"
branch_labels = None
depends_on = None


def _colunas_existentes(tabela: str) -> set[str]:
    bind = op.get_bind()
    insp = sa.inspect(bind)
    return {col["name"] for col in insp.get_columns(tabela)}


def upgrade():
    existentes = _colunas_existentes("saves")
    with op.batch_alter_table("saves", schema=None) as batch_op:
        if "metadados" not in existentes:
            batch_op.add_column(
                sa.Column(
                    "metadados", sa.JSON(), server_default=sa.text("'{}'"), nullable=True
                )
            )
        if "save_version" not in existentes:
            batch_op.add_column(
                sa.Column(
                    "save_version",
                    sa.Integer(),
                    server_default=sa.text("1"),
                    nullable=False,
                )
            )
        if "checksum" not in existentes:
            batch_op.add_column(
                sa.Column("checksum", sa.String(length=64), nullable=True)
            )
        if "criado_em" not in existentes:
            batch_op.add_column(
                sa.Column(
                    "criado_em",
                    sa.DateTime(timezone=True),
                    server_default=sa.text("(CURRENT_TIMESTAMP)"),
                    nullable=True,
                )
            )
        if "atualizado_em" not in existentes:
            batch_op.add_column(
                sa.Column(
                    "atualizado_em",
                    sa.DateTime(timezone=True),
                    server_default=sa.text("(CURRENT_TIMESTAMP)"),
                    nullable=True,
                )
            )


def downgrade():
    existentes = _colunas_existentes("saves")
    with op.batch_alter_table("saves", schema=None) as batch_op:
        for col in ("metadados", "save_version", "checksum", "criado_em"):
            if col in existentes:
                batch_op.drop_column(col)

