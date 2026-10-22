"""The local demo must be useful, repeatable, and isolated from normal data."""

from datetime import datetime, timedelta, timezone
import sqlite3

import pytest

from app import create_app
from app.domain.enums import MatchStatus, RegistrationStatus, WinnerSide
from app.services.dual_write import get_match_player_ids
from app.services.permission_service import PermissionService
from app.services.registration_resolver import team_registrations_for_tournament
from models import (
    League,
    Match,
    Player,
    PlayerRegistration,
    Point,
    SideComp,
    SideCompRegistration,
    Team,
    TeamRegistration,
    Tournament,
    db,
)
from scripts import dev_demo


@pytest.fixture
def demo_path(tmp_path):
    path = tmp_path / "instance" / "demo.db"
    assert dev_demo.prepare_demo_database(path, now=datetime(2026, 10, 22, 18, tzinfo=timezone.utc))
    return path


@pytest.fixture
def demo_app(demo_path):
    app = create_app({"TESTING": True, "SQLALCHEMY_DATABASE_URI": f"sqlite:///{demo_path}"})
    with app.app_context():
        yield app
        db.session.remove()
        db.engine.dispose()


def test_reseeding_preserves_edits_and_does_not_touch_other_database(demo_path):
    protected = demo_path.with_name("tournament.db")
    protected.write_bytes(b"existing tournament data must not change")
    with sqlite3.connect(demo_path) as connection:
        connection.execute("UPDATE tournaments SET name = 'My local changes' WHERE url = ?", (dev_demo.LIVE_EVENT,))
        before = connection.execute("SELECT COUNT(*) FROM players").fetchone()[0]
    snapshot = demo_path.read_bytes()
    assert dev_demo.prepare_demo_database(demo_path) is False
    assert demo_path.read_bytes() == snapshot
    assert protected.read_bytes() == b"existing tournament data must not change"
    with sqlite3.connect(demo_path) as connection:
        assert connection.execute("SELECT COUNT(*) FROM players").fetchone()[0] == before
        assert (
            connection.execute("SELECT name FROM tournaments WHERE url = ?", (dev_demo.LIVE_EVENT,)).fetchone()[0]
            == "My local changes"
        )


def test_seed_has_valid_relationships_permissions_and_match_state(demo_app):
    assert Player.query.count() == 6
    assert Team.query.count() == 4
    assert Tournament.query.count() == 4
    assert League.query.count() == 1
    assert db.session.execute(db.text("PRAGMA foreign_key_check")).all() == []
    organizer = db.session.get(Player, dev_demo.ORGANIZER_ID)
    assert organizer.check_password(dev_demo.DEMO_PASSWORD)
    assert db.session.get(Player, dev_demo.PLAYER_ID).check_password(dev_demo.DEMO_PASSWORD)
    assert db.session.get(Team, dev_demo.TEAM_ID).check_password(dev_demo.DEMO_PASSWORD)
    for tournament in Tournament.query.all():
        assert tournament.published and tournament.schedule_published
        assert PermissionService.is_tournament_organizer(tournament.url, organizer)
        assert PermissionService.can_head_ref_match(tournament.url, organizer)
        assert (tournament.league_id is None) != (tournament.registrable_config_id is None)
    league_event = db.session.get(Tournament, dev_demo.LEAGUE_EVENT)
    assert len(team_registrations_for_tournament(league_event)) == 4
    assert TeamRegistration.query.filter_by(event=dev_demo.LEAGUE_EVENT).count() == 0
    pending = PlayerRegistration.query.filter_by(event=dev_demo.LIVE_EVENT, player=dev_demo.PLAYER_ID).one()
    assert pending.status == RegistrationStatus.PENDING_TEAM_APPROVAL and not pending.paid
    assert {match.status for match in Match.query.all()} >= {MatchStatus.IN_PROGRESS, MatchStatus.COMPLETED}
    for match in Match.query.filter(Match.status.in_([MatchStatus.IN_PROGRESS, MatchStatus.COMPLETED])):
        team1 = set(get_match_player_ids(match, WinnerSide.TEAM1))
        team2 = set(get_match_player_ids(match, WinnerSide.TEAM2))
        assert len(team1) == len(team2) == 2
        assert not team1.intersection(team2)
        points = Point.query.filter_by(match=match.uuid).order_by(Point.stamp).all()
        assert sum(point.nstones for point in points) + match.stones_remaining == match.stones_per_set
        assert all(point.end_stamp - point.stamp == timedelta(seconds=point.nstones * 1.5) for point in points)
        if match.status == MatchStatus.COMPLETED:
            assert match.match_winner == WinnerSide.TEAM1 and match.finalized_at
    competition = SideComp.query.one()
    assert competition.registration_open and competition.active
    entries = SideCompRegistration.query.filter_by(comp=competition.id).all()
    assert len(entries) == 3 and len({entry.entry_number for entry in entries}) == 3


def test_demo_api_authenticates_both_account_types(demo_app):
    client = demo_app.test_client()
    for username, expected_type in [(dev_demo.ORGANIZER_ID, "player"), (dev_demo.TEAM_ID, "team")]:
        response = client.post("/_api/login", json={"username": username, "password": dev_demo.DEMO_PASSWORD})
        assert response.status_code == 200, response.get_json()
        user = client.get("/_api/me").get_json()
        assert user["id"] == username and user["type"] == expected_type
        client.post("/_api/logout")


def test_demo_info_is_only_registered_by_demo_launcher(demo_path, demo_app):
    assert demo_app.test_client().get("/_api/demo-info").status_code == 404
    isolated_app = dev_demo.create_demo_app(demo_path)
    try:
        response = isolated_app.test_client().get("/_api/demo-info")
        assert response.status_code == 200
        assert response.get_json() == {"demo": True, "organizer": dev_demo.ORGANIZER_ID}
        assert demo_app.test_client().get("/_api/demo-info").status_code == 404
    finally:
        with isolated_app.app_context():
            db.session.remove()
            db.engine.dispose()


def test_demo_rejects_production_names_and_symlinks(tmp_path):
    production = tmp_path / "tournament.db"
    production.write_bytes(b"do not modify")
    with pytest.raises(ValueError):
        dev_demo.prepare_demo_database(production)
    symlink = tmp_path / "demo.db"
    symlink.symlink_to(production)
    with pytest.raises(ValueError):
        dev_demo.prepare_demo_database(symlink)
    assert production.read_bytes() == b"do not modify"


def test_seed_only_never_runs_the_server(tmp_path, monkeypatch, capsys):
    path = tmp_path / "instance" / "demo.db"
    monkeypatch.setattr(dev_demo, "DEFAULT_DATABASE", path)
    monkeypatch.setattr(dev_demo, "prepare_demo_database", lambda: True)
    monkeypatch.setattr(dev_demo.Flask, "run", lambda *args, **kwargs: pytest.fail("Server started"))
    dev_demo.main(["--seed-only"])
    assert "Created sample data" in capsys.readouterr().out
