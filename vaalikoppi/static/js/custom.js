
// Kaikenlaisia funktioita äänestykseen
// Tarvii jQueryn


SITE_ROOT_PATH = '/vaalikoppi/';

function vote(votingId) {

	var form = $('#voting-form-' + votingId);
	var maxVotes;
	var chosenCandidates = [];

	try {
		maxVotes = parseInt(form.attr('data-voting-max-votes'));
	} catch (err) {
		alert('Äänestyksen tiedot eivät ole latautuneet oikein. Päivitä sivu.');
		return;
	}

	form.find('input[name=candidate]:checked').each(function() {
		var curId = $(this).attr('value');
		var curName = form.find('label[for=candidate-v-' + votingId + '-' + curId + ']').text();
		chosenCandidates.push({'id' : curId, 'name' : curName});
	});

	var confirmation = confirm('Olet äänestämässä ' + (chosenCandidates.length > 1 ? 'ehdokkaita:\n' : 'ehdokasta:\n' ) +
	chosenCandidates.map(function(candi) {
		return candi.name;
	})
	.join(', '));

	if (!confirmation) {
		return;
	}

	form.find('input, button').prop('disabled', true);

	var query = $.post(SITE_ROOT_PATH + 'votings/' + votingId + '/vote/',
		{
			candidates : chosenCandidates.map(function(candi) {
				return candi.id;
			})
		}
	).done(function(data) {
		$('#voting-list-area').html(data);
	}).fail(function(data) {
		alert('Äänestäminen epäonnistui. Päivitä sivu ja yritä uudelleen!');
		refreshVotingList();
	});

}

function refreshVotingList(admin = false) {

	var votingArea = $('#voting-list-area');

	var query = $.get(SITE_ROOT_PATH + (admin ? 'admin/' : '' ) + 'votings/list/', function(data) {
		votingArea.html(data);
	})
	.fail(function() {
		alert('Äänestysten haku ei onnistunut. Päivitä sivu. Jos koetit äänestää, katso, näkyykö äänestys jo äänestettynä.');
	});
}

function checkboxClick(votingId, candidateId) {

	var form = $('#voting-form-' + votingId);
	var maxVotes = parseInt(form.attr('data-voting-max-votes'));

	if (form.find('input[type=checkbox]:checked').length > maxVotes) {
		form.find('#candidate-v-' + votingId + '-'+ candidateId).prop('checked', false);
	}

}

function generateTokens(count) {

	$('#generate-tokens-button').prop('disabled', true);

	var query = $.post(SITE_ROOT_PATH + 'admin/tokens/generate/',
		{ count : count }
	).done(function(data) {
		alert('Koodien generointi onnistui.');
		location.reload();
	}).fail(function(data) {
		alert('Koodien generointi epäonnistui.');
	});

	$('#generate-tokens-button').prop('disabled', false);
}

function invalidateToken(code, number) {

	if (confirm('Haluatko varmasti mitätöidä koodin ' + code +'? Tehtyä mitätöintiä ei voi peruuttaa.')) {

		var token = code;

		if (token.length < 1) {
			return;
		}

		$('#invalidate-token-button-'+number).prop('disabled', true);

		var query = $.post(SITE_ROOT_PATH + 'admin/tokens/invalidate/',
			{ token : token }
		).done(function(data) {
			alert('Koodin invalidointi onnistui.');
			location.reload();
		}).fail(function(data) {
			alert('Koodin invalidointi epäonnistui. Tarkista koodi.');
		});

		$('#invalidate-token-button-'+number).prop('disabled', false);

	} else {
			return;
	}
}

function activateToken(code, number) {

	if (confirm('Haluatko varmasti aktivoida koodin ' + code +'?')) {
		var token = code;

		if (token.length < 1) {
			return;
		}

		$('#activate-token-button-'+number).prop('disabled', true);

		var query = $.post(SITE_ROOT_PATH + 'admin/tokens/activate/',
			{ token : token }
		).done(function(data) {
			alert('Koodin aktivointi onnistui.');
			location.reload();
		}).fail(function(data) {
			alert('Koodin aktivointi epäonnistui. Tarkista koodi.');
		});

		$('#activate-token-button-'+number).prop('disabled', false);
	} else {
			return;
	}
}

// Just for the UI. Everything is validated in the back-end...
function checkVoterStatus(callback) {
	var query = $.getJSON(SITE_ROOT_PATH + 'user/status/')
	.done(function(data) {
		try {
			// No token/non-active token
			if (data.status === 0) {
				callback(false);
			} else if (data.status === 1) {
				callback(true)
			} else {
				throw new Exception();
			}
		} catch (err) {
			callback(false);
		}
	})
	.fail(function() {
		alert('Tilan haku palvelimelta ei onnistunut. Koeta päivittää sivu.');
	});
}

function submitToken() {

	var token = $('#type-token-field').val();
	var warning = $('#main-login-prompt .wrong-token-warning');

	warning.addClass('invisible');

	var query = $.post(SITE_ROOT_PATH + 'user/login/',
		{ token : token }
	).done(function(data) {
		toggleLoginPrompt();
		refreshVotingList();
	}).fail(function(data) {
		window.setTimeout(function() {
			warning.removeClass('invisible');
		}, 100);
	});
}

function toggleLoginPrompt() {
	$('#main-login-prompt').toggle();
	$('#voting-list-updater, #voting-list-area').fadeToggle();
}

function add_candidate(voting_id) {
	var form = $('#candidate_name');
	var candidate_name = $('#candidate_name').val();

	var query = $.post(SITE_ROOT_PATH + 'admin/votings/' + voting_id + '/add/', { candidate_name: candidate_name }).done(function(data) {
		refreshVotingList(true);
	}).fail(function(data) {
		alert('Ehdokkaan lisääminen ei ehkä onnistunut! Päivitä sivu!');
	});
}

function remove_candidate(candidate_id) {
	var query = $.post(SITE_ROOT_PATH + 'admin/votings/' + candidate_id + '/remove/').done(function(data) {
		refreshVotingList(true);
	}).fail(function(data) {
		alert('Äänestyksen luominen ei ehkä onnistunut! Päivitä sivu!');
	});
}

function closeVoting(votingId) {

	var query = $.post(SITE_ROOT_PATH + 'admin/votings/' + votingId + '/close/').done(function(data) {
		refreshVotingList(true);
	}).fail(function(data) {
		alert('Äänestyksen sulkeminen ei ehkä onnistunut! Päivitä sivu!');
	});
}

function openVoting(votingId) {

	var query = $.post(SITE_ROOT_PATH + 'admin/votings/' + votingId + '/open/').done(function(data) {
		refreshVotingList(true);
	}).fail(function(data) {
		alert('Äänestyksen avaaminen ei ehkä onnistunut! Päivitä sivu!');
	});
}

// Hakutaulukon funktioi

function searchFunction() {
  // Declare variables
  var input, filter, table, tr, td, i;
  input = document.getElementById("searchInput");
  filter = input.value.toUpperCase();
  table = document.getElementById("searchTable");
  tr = table.getElementsByTagName("tr");

  // Loop through all table rows, and hide those who don't match the search query
  for (i = 0; i < tr.length; i++) {
    td = tr[i].getElementsByTagName("td")[0];
    if (td) {
      if (td.innerHTML.toUpperCase().indexOf(filter) > -1) {
        tr[i].style.display = "";
      } else {
        tr[i].style.display = "none";
      }
    }
  }
}
